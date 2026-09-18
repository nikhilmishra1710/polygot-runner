package api

import (
	"context"
	"encoding/json"
	"net/http"

	"github.com/gorilla/websocket"
	pb "runtime-platform/api/gen/execution/v1"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

type Handler struct {
	workerClient pb.ExecutionServiceClient
	manager      *ExecutionManager
}

func NewHandler(wc pb.ExecutionServiceClient, manager *ExecutionManager) *Handler {
	return &Handler{
		workerClient: wc,
		manager:      manager,
	}
}

// POST /v1/executions
func (h *Handler) CreateExecution(w http.ResponseWriter, r *http.Request) {
	var req pb.ExecuteRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request JSON: "+err.Error(), http.StatusBadRequest)
		return
	}

	// Use context.Background() so the job survives after the HTTP request finishes
	jobID, execCtx := h.manager.Start(context.Background(), &req)

	go func() {
		defer h.manager.Cleanup(jobID)

		h.manager.Store.UpdateState(context.Background(), jobID, StateRunning, nil)
		stream, err := h.workerClient.Execute(execCtx, &req)
		if err != nil {
			h.manager.Store.UpdateState(context.Background(), jobID, StateFailed, nil)
			return
		}

		for {
			event, err := stream.Recv()
			if err != nil {
				break
			}

			h.manager.RouteEvent(jobID, event)

			if event.Type == pb.ExecutionEvent_FINISHED {
				h.manager.Store.UpdateState(context.Background(), jobID, StateCompleted, event.FinalResult)
			}
		}
	}()

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	json.NewEncoder(w).Encode(map[string]string{"id": jobID})
}

// GET /v1/executions/{id}
func (h *Handler) GetExecution(w http.ResponseWriter, r *http.Request, id string) {
	exec, err := h.manager.Store.Get(r.Context(), id)
	if err != nil {
		http.Error(w, "Execution not found", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"id":         exec.ID,
		"state":      exec.State,
		"created_at": exec.CreatedAt,
		"result":     exec.Result,
	})
}

// GET /v1/executions/{id}/stream (Upgrades to WebSocket)
func (h *Handler) StreamExecution(w http.ResponseWriter, r *http.Request, id string) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	history, _ := h.manager.Store.GetEvents(r.Context(), id)
	for _, event := range history {
		if err := conn.WriteJSON(event); err != nil {
			return
		}
	}

	exec, err := h.manager.Store.Get(r.Context(), id)
	if err != nil {
		conn.WriteJSON(map[string]string{"error": "Execution not found"})
		return
	}

	if exec.State == StateRunning || exec.State == StateCreated {
		liveEvents := h.manager.Subscribe(id)
		for event := range liveEvents {
			if err := conn.WriteJSON(event); err != nil {
				break
			}
		}
	}

	conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
}

// POST /v1/executions/{id}/cancel
func (h *Handler) CancelExecution(w http.ResponseWriter, r *http.Request, id string) {
	h.manager.Cancel(id)
	w.WriteHeader(http.StatusNoContent)
}
