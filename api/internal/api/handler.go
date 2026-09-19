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

	jobID, execCtx := h.manager.Start(context.Background(), &req)

	go func() {
		defer func() {
			// Wake up any waiting WebSockets one final time before closing
			h.manager.mu.Lock()
			for _, ch := range h.manager.listeners[jobID] {
				select {
				case ch <- struct{}{}:
				default:
				}
			}
			h.manager.mu.Unlock()

			h.manager.Cleanup(jobID)
		}()

		// The Store safely ignores this if the job was already CANCELLED
		h.manager.Store.UpdateState(context.Background(), jobID, StateRunning, nil)

		stream, err := h.workerClient.Execute(execCtx, &req)
		if err != nil {
			h.manager.Store.UpdateState(context.Background(), jobID, StateFailed, nil)
			return
		}

		for {
			event, err := stream.Recv()
			if err != nil {
				// Context canceled or stream broken
				h.manager.Store.UpdateState(context.Background(), jobID, StateFailed, nil)
				break
			}

			if event.Type == pb.ExecutionEvent_FINISHED {
				h.manager.Store.UpdateState(context.Background(), jobID, StateCompleted, event.FinalResult)
			}

			h.manager.RouteEvent(jobID, event)
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

	// Register this specific client connection for wakeups
	wakeup := h.manager.Subscribe(id)
	defer h.manager.Unsubscribe(id, wakeup)

	cursor := 0

	for {
		// 1. Fetch the unified timeline from the store
		events, err := h.manager.Store.GetEvents(r.Context(), id)
		if err != nil {
			conn.WriteJSON(map[string]string{"error": "Execution not found"})
			return
		}

		// 2. Play back any events we haven't sent to this client yet
		for i := cursor; i < len(events); i++ {
			if err := conn.WriteJSON(events[i]); err != nil {
				return // Client disconnected
			}
			cursor++
		}

		// 3. Check if the job is finished
		exec, _ := h.manager.Store.Get(r.Context(), id)
		if exec.State != StateRunning && exec.State != StateCreated {
			break // Exit the loop gracefully
		}

		// 4. Block until the manager signals new data, or the client disconnects
		<-wakeup
	}

	conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
}

// POST /v1/executions/{id}/cancel
func (h *Handler) CancelExecution(w http.ResponseWriter, r *http.Request, id string) {
	// 1. Explicitly lock the state as CANCELLED first!
	// Our strict Store rules will now prevent the dying goroutine from changing this to FAILED.
	h.manager.Store.UpdateState(context.Background(), id, StateCancelled, nil)
	
	// 2. Kill the Rust gRPC process
	h.manager.Cancel(id)
	
	w.WriteHeader(http.StatusNoContent)
}