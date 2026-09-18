package api

import (
	"encoding/json"
	"github.com/gorilla/websocket"
	"io"
	"log"
	"net/http"
	pb "runtime-platform/api/gen/execution/v1"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true }, // Restrict in production
}

type Handler struct {
	workerClient pb.ExecutionServiceClient
}

func NewHandler(wc pb.ExecutionServiceClient) *Handler {
	return &Handler{workerClient: wc}
}

func (h *Handler) StreamHandler(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	// 1. Read the ExecuteRequest from the browser
	var req pb.ExecuteRequest
	if err := conn.ReadJSON(&req); err != nil {
		conn.WriteMessage(websocket.TextMessage, []byte("Invalid request JSON"))
		return
	}

	// (Optional) Call your existing ValidateExecutionRequest(&req) here

	// 2. Open the gRPC stream to Rust
	stream, err := h.workerClient.Execute(r.Context(), &req)
	if err != nil {
		conn.WriteMessage(websocket.TextMessage, []byte("Worker unavailable"))
		return
	}

	// 3. Pipe gRPC events directly to the WebSocket
	for {
		event, err := stream.Recv()
		if err == io.EOF {
			break // Rust cleanly closed the stream
		}
		if err != nil {
			log.Printf("gRPC stream error: %v", err)
			break
		}

		if err := conn.WriteJSON(event); err != nil {
			log.Printf("Client disconnected: %v", err)
			break
		}
	}

	// Close cleanly
	conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
}

// SyncHandler provides a standard REST interface that waits for the execution to finish
func (h *Handler) SyncHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// 1. Parse the incoming JSON into the protobuf request
	var req pb.ExecuteRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request JSON", http.StatusBadRequest)
		return
	}

	// 2. Open the gRPC stream to the Rust worker
	stream, err := h.workerClient.Execute(r.Context(), &req)
	if err != nil {
		http.Error(w, "Worker unavailable: "+err.Error(), http.StatusServiceUnavailable)
		return
	}

	var finalResult *pb.JobResult

	// 3. Consume the stream until we get the final result or an EOF
	for {
		event, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			http.Error(w, "Execution stream failed: "+err.Error(), http.StatusInternalServerError)
			return
		}

		// We only care about the FINISHED event for this synchronous endpoint
		if event.Type == pb.ExecutionEvent_FINISHED {
			finalResult = event.FinalResult
		}
	}

	// 4. Return the final result as JSON
	if finalResult == nil {
		http.Error(w, "Execution finished without a result", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(finalResult)
}
