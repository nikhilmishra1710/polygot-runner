package api

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"

	"runtime-platform/api/internal/worker"
)

// ExecutionRequest represents the incoming HTTP request for execution
type ExecutionRequest struct {
	Language string `json:"language"`
	Files    []File `json:"files"`
	Stdin    string `json:"stdin"`
}

// File represents a file in the execution request
type File struct {
	Path     string `json:"path"`
	Contents string `json:"contents"`
}

// ExecutionResponse represents the HTTP response for execution
type ExecutionResponse struct {
	ID      string `json:"id"`
	Status  string `json:"status"`
	Stdout  string `json:"stdout"`
	Stderr  string `json:"stderr"`
	ExitCode int    `json:"exit_code"`
}

// handleExecutions handles POST /v1/executions
func (s *Server) handleExecutions(w http.ResponseWriter, r *http.Request) {
	// Decode request body
	var req ExecutionRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON: "+err.Error(), http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	// Validate request
	if err := validateExecutionRequest(&req); err != nil {
		http.Error(w, "Validation error: "+err.Error(), http.StatusBadRequest)
		return
	}

	// Convert to worker request
	workerReq := convertToWorkerRequest(&req)

	// Execute via worker client (with timeout)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	result, err := s.workerClient.Execute(ctx, workerReq)
	if err != nil {
		log.Printf("Worker execution failed: %v", err)
		http.Error(w, "Internal server error: "+err.Error(), http.StatusInternalServerError)
		return
	}

	// Convert worker result to HTTP response
	resp := convertToHTTPResponse(result)

	// Respond with JSON
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

// validateExecutionRequest validates the incoming request
func validateExecutionRequest(req *ExecutionRequest) error {
	if req.Language == "" {
		return fmt.Errorf("language is required")
	}
	if len(req.Files) == 0 {
		return fmt.Errorf("at least one file is required")
	}
	for _, f := range req.Files {
		if f.Path == "" {
			return fmt.Errorf("file path is required")
		}
		if f.Contents == "" {
			return fmt.Errorf("file contents is required for path: %s", f.Path)
		}
	}
	return nil
}

// convertToWorkerRequest converts HTTP request to worker request
func convertToWorkerRequest(req *ExecutionRequest) worker.ExecutionRequest {
	// In a real implementation, this would map to the worker's protobuf or internal types
	// For now, we'll create a simple mapping
	workerFiles := make([]worker.File, len(req.Files))
	for i, f := range req.Files {
		workerFiles[i] = worker.File{
			Path:     f.Path,
			Contents: f.Contents,
		}
	}
	return worker.ExecutionRequest{
		Language: req.Language,
		Files:    workerFiles,
		Stdin:    req.Stdin,
	}
}

// convertToHTTPResponse converts worker result to HTTP response
func convertToHTTPResponse(result worker.ExecutionResult) ExecutionResponse {
	return ExecutionResponse{
		ID:      result.ID,
		Status:  result.Status,
		Stdout:  result.Stdout,
		Stderr:  result.Stderr,
		ExitCode: result.ExitCode,
	}
}