// Package tests contains end-to-end integration tests:
//
//	Go gRPC Client → Rust workerd (TCP 50051) → Worker → ExecutionEngine → Sandbox
//
// Nothing is mocked. Tests that require the sandboxed worker need a
// built workerd binary and root (namespaces/cgroups); they skip otherwise.
// Pure validation tests run directly against the Go API layer.
package tests

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"

	pb "runtime-platform/api/gen/execution/v1"
	"runtime-platform/api/internal/api"
)

// clientFor returns a gRPC client connected directly to the Rust worker.
func clientFor(t *testing.T, addr string) pb.ExecutionServiceClient {
	t.Helper()
	// Dial the Rust gRPC server
	conn, err := grpc.Dial(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("dial worker: %v", err)
	}
	t.Cleanup(func() { conn.Close() })

	return pb.NewExecutionServiceClient(conn)
}

func workerdBin(t *testing.T) string {
	t.Helper()
	if p := os.Getenv("WORKERD_BIN"); p != "" {
		return p
	}
	p := filepath.Join("..", "..", "runtime", "target", "debug", "workerd")
	if _, err := os.Stat(p); err != nil {
		t.Skipf("workerd binary not found at %s (set WORKERD_BIN)", p)
	}
	return p
}

func startWorkerd(t *testing.T) string {
	t.Helper()
	if os.Geteuid() != 0 {
		t.Skip("requires root: sandbox uses namespaces and cgroups")
	}
	bin := workerdBin(t)
	targetAddr := "127.0.0.1:50051"

	// Actively poll the OS until the port is completely released from TIME_WAIT
	for {
		l, err := net.Listen("tcp", targetAddr)
		if err == nil {
			l.Close()
			break
		}
		time.Sleep(100 * time.Millisecond)
	}

	cmd := exec.Command(bin)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Start(); err != nil {
		t.Fatalf("start workerd: %v", err)
	}
	t.Cleanup(func() {
		_ = cmd.Process.Kill()
		_ = cmd.Wait()
	})

	// Wait for the TCP port to open
	deadline := time.Now().Add(5 * time.Second)
	for {
		conn, err := net.DialTimeout("tcp", targetAddr, 100*time.Millisecond)
		if err == nil {
			conn.Close()
			break
		}
		if time.Now().After(deadline) {
			t.Fatalf("workerd did not bind to %s", targetAddr)
		}
		time.Sleep(50 * time.Millisecond)
	}

	return targetAddr
}

// --- End-to-end (real workerd) ---

func TestExecutePythonSuccess(t *testing.T) {
	addr := startWorkerd(t)
	client := clientFor(t, addr)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	stream, err := client.Execute(ctx, &pb.ExecuteRequest{
		Language: "python",
		Files: []*pb.SourceFile{
			{Path: "main.py", Contents: []byte(`print("Hello")`)},
		},
	})
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}

	var stdout []byte
	var finalResult *pb.JobResult

	// Read from the gRPC stream
	for {
		event, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("stream error: %v", err)
		}

		if event.Type == pb.ExecutionEvent_STDOUT {
			stdout = append(stdout, event.Data...)
		} else if event.Type == pb.ExecutionEvent_FINISHED {
			finalResult = event.FinalResult
		}
	}

	if finalResult == nil {
		t.Fatalf("Expected final result, but stream closed early")
	}

	if string(stdout) != "Hello\n" {
		t.Errorf("stdout = %q, want %q", string(stdout), "Hello\n")
	}

	term := finalResult.GetReport().GetTermination()
	if term.GetExitCode() != 0 {
		t.Errorf("exit_code = %v, want 0", term.GetExitCode())
	}
}

func TestExecutePythonRuntimeError(t *testing.T) {
	addr := startWorkerd(t)
	client := clientFor(t, addr)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	stream, err := client.Execute(ctx, &pb.ExecuteRequest{
		Language: "python",
		Files: []*pb.SourceFile{
			{Path: "main.py", Contents: []byte(`raise RuntimeError("boom")`)},
		},
	})
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}

	var stderr []byte
	var finalResult *pb.JobResult

	for {
		event, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("stream error: %v", err)
		}

		if event.Type == pb.ExecutionEvent_STDERR {
			stderr = append(stderr, event.Data...)
		} else if event.Type == pb.ExecutionEvent_FINISHED {
			finalResult = event.FinalResult
		}
	}

	if finalResult == nil {
		t.Fatalf("Expected final result, but stream closed early")
	}

	term := finalResult.GetReport().GetTermination()
	if term.GetExitCode() == 0 {
		t.Errorf("exit_code = 0, want non-zero")
	}

	if !strings.Contains(string(stderr), "boom") {
		t.Errorf("stderr = %q, want to contain traceback with boom", string(stderr))
	}
}

// --- workerd unavailable ---

func TestWorkerUnavailable(t *testing.T) {
	// Point to a local port that we know is completely dead
	client := clientFor(t, "127.0.0.1:50052")

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	stream, err := client.Execute(ctx, &pb.ExecuteRequest{
		Language: "python",
		Files:    []*pb.SourceFile{{Path: "main.py", Contents: []byte("print(1)")}},
	})
	if err != nil {
		// Sometimes gRPC fails on dial if it knows the port is dead
		requireCode(t, err, codes.Unavailable)
		return
	}

	// Otherwise, it fails lazily on the first stream read
	_, err = stream.Recv()
	requireCode(t, err, codes.Unavailable)
}

func requireCode(t *testing.T, err error, want codes.Code) {
	t.Helper()
	if err == nil {
		t.Fatalf("expected error with code %v, got nil", want)
	}
	if got := status.Code(err); got != want {
		t.Fatalf("error code = %v, want %v (err: %v)", got, want, err)
	}
}

// --- Validation: rejected at the API boundary, worker never consulted ---

func TestUnsupportedLanguage(t *testing.T) {
	req := &pb.ExecuteRequest{
		Language: "java",
		Files:    []*pb.SourceFile{{Path: "Main.java", Contents: []byte("class Main {}")}},
	}
	err := api.ValidateExecutionRequest(req)
	requireValidationError(t, err, "unsupported language")
}

func TestEmptyFiles(t *testing.T) {
	req := &pb.ExecuteRequest{Language: "python"}
	err := api.ValidateExecutionRequest(req)
	requireValidationError(t, err, "at least one file")
}

func TestInvalidFilePath(t *testing.T) {
	req := &pb.ExecuteRequest{
		Language: "python",
		Files:    []*pb.SourceFile{{Path: "../evil.py", Contents: []byte("pass")}},
	}
	err := api.ValidateExecutionRequest(req)
	requireValidationError(t, err, "..")
}

func TestSourceTooLarge(t *testing.T) {
	big := make([]byte, 256*1024+1)
	req := &pb.ExecuteRequest{
		Language: "python",
		Files:    []*pb.SourceFile{{Path: "big.py", Contents: big}},
	}
	err := api.ValidateExecutionRequest(req)
	requireValidationError(t, err, "too large")
}

func requireValidationError(t *testing.T, err error, want string) {
	t.Helper()
	if err == nil {
		t.Fatalf("expected error containing %q, got nil", want)
	}
	if !strings.Contains(err.Error(), want) {
		t.Fatalf("expected error to contain %q, got %q", want, err.Error())
	}
}

func TestCancellationKillsProcessAndFreesThreads(t *testing.T) {
	addr := startWorkerd(t)
	client := clientFor(t, addr)

	var cancels []context.CancelFunc

	// 1. Exhaust all 4 worker threads with a continuously ticking job
	for i := 0; i < 4; i++ {
		ctx, cancel := context.WithCancel(context.Background())
		cancels = append(cancels, cancel)

		stream, err := client.Execute(ctx, &pb.ExecuteRequest{
			Language: "python",
			Files: []*pb.SourceFile{
				// We tick every 100ms. This forces Rust to check the gRPC connection state constantly.
				{Path: "main.py", Contents: []byte(`import time, sys
for _ in range(100):
    print("tick", flush=True)
    time.sleep(0.1)`)},
			},
		})
		if err != nil {
			t.Fatalf("Execute failed: %v", err)
		}

		// Wait for the STARTED event to ensure the thread is actively blocked
		event, err := stream.Recv()
		if err != nil || event.Type != pb.ExecutionEvent_STARTED {
			t.Fatalf("Failed to start long job %d", i)
		}
	}

	// 2. Cancel all 4 streams! This mimics the WebSocket disconnecting.
	for _, cancel := range cancels {
		cancel()
	}

	// Give Rust a brief moment to process the HTTP/2 RST_STREAM and kill the sandboxes.
	// Since our script ticks every 100ms, 200ms is plenty of time to catch the drop.
	time.Sleep(200 * time.Millisecond)

	// 3. Send a 5th quick job with a strict 2-second timeout
	ctx5, cancel5 := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel5()

	stream5, err := client.Execute(ctx5, &pb.ExecuteRequest{
		Language: "python",
		Files: []*pb.SourceFile{
			{Path: "main.py", Contents: []byte(`print("I am freee!")`)},
		},
	})
	if err != nil {
		t.Fatalf("5th job failed to connect: %v", err)
	}

	var stdout []byte
	for {
		event, err := stream5.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("5th job stream failed (cancellation is likely broken!): %v", err)
		}
		if event.Type == pb.ExecutionEvent_STDOUT {
			stdout = append(stdout, event.Data...)
		}
	}

	if string(stdout) != "I am freee!\n" {
		t.Errorf("Unexpected output from 5th job: %s", stdout)
	}
}

func TestGoAPIConcurrentWebSockets(t *testing.T) {
	// 1. Start the real Rust worker
	addr := startWorkerd(t)
	grpcClient := clientFor(t, addr)

	// 2. Initialize the new state manager and store
	store := api.NewInMemoryStore()
	manager := api.NewExecutionManager(store)
	handler := api.NewHandler(grpcClient, manager)

	// 3. Start the Go API layer in a test HTTP server with the new REST routing
	mux := http.NewServeMux()

	// Bind POST /v1/executions
	mux.HandleFunc("/v1/executions", handler.CreateExecution)

	// Bind GET (WS) /v1/executions/{id}/stream
	mux.HandleFunc("/v1/executions/", func(w http.ResponseWriter, r *http.Request) {
		path := strings.TrimPrefix(r.URL.Path, "/v1/executions/")
		parts := strings.Split(path, "/")
		if len(parts) == 2 && parts[1] == "stream" {
			handler.StreamExecution(w, r, parts[0])
		} else {
			http.Error(w, "Not found", http.StatusNotFound)
		}
	})

	server := httptest.NewServer(mux)
	defer server.Close()

	var wg sync.WaitGroup
	startTime := time.Now()

	// 4. Define the concurrent worker function
	runClient := func(identity string) {
		defer wg.Done()

		script := `
import time, sys
print("START ` + identity + `")
sys.stdout.flush()
time.sleep(0.5)
print("MID ` + identity + `")
sys.stdout.flush()
time.sleep(0.5)
print("END ` + identity + `")
`
		req := &pb.ExecuteRequest{
			Language: "python",
			Files: []*pb.SourceFile{
				{Path: "main.py", Contents: []byte(script)},
			},
		}

		// Step A: POST to /v1/executions to start the job
		reqBody, _ := json.Marshal(req)
		resp, err := http.Post(server.URL+"/v1/executions", "application/json", bytes.NewBuffer(reqBody))
		if err != nil {
			t.Errorf("Client %s failed to POST: %v", identity, err)
			return
		}
		defer resp.Body.Close()

		var result map[string]string
		if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
			t.Errorf("Client %s failed to decode response: %v", identity, err)
			return
		}
		jobID := result["id"]

		// Step B: Connect to the WebSocket stream using the new WS path
		wsURL := "ws" + strings.TrimPrefix(server.URL, "http") + "/v1/executions/" + jobID + "/stream"
		conn, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
		if err != nil {
			t.Errorf("Client %s failed to connect WS: %v", identity, err)
			return
		}
		defer conn.Close()

		var stdout []byte

		// Read the live stream from Go
		for {
			_, msg, err := conn.ReadMessage()
			if err != nil {
				if websocket.IsUnexpectedCloseError(err, websocket.CloseNormalClosure) {
					t.Errorf("Client %s unexpected close: %v", identity, err)
				}
				break
			}

			var event map[string]interface{}
			if err := json.Unmarshal(msg, &event); err != nil {
				continue
			}

			// Check for STDOUT events (Type 1)
			if evtType, ok := event["type"].(float64); ok && evtType == 1 {
				if dataStr, ok := event["data"].(string); ok {
					decoded, _ := base64.StdEncoding.DecodeString(dataStr)
					stdout = append(stdout, decoded...)
				}
			}
		}

		expected := "START " + identity + "\nMID " + identity + "\nEND " + identity + "\n"
		if string(stdout) != expected {
			t.Errorf("Client %s output mismatch. Got:\n%s\nExpected:\n%s", identity, string(stdout), expected)
		}
	}

	// 5. Fire off 3 concurrent clients
	wg.Add(3)
	go runClient("A")
	go runClient("B")
	go runClient("C")

	wg.Wait()
	elapsed := time.Since(startTime)

	if elapsed > 2500*time.Millisecond {
		t.Errorf("Go API serialized the requests! Total time: %v", elapsed)
	}
}

func TestExecutionLifecycle_StatesAndCancellation(t *testing.T) {
	addr := startWorkerd(t)
	grpcClient := clientFor(t, addr)

	store := api.NewInMemoryStore()
	manager := api.NewExecutionManager(store)
	handler := api.NewHandler(grpcClient, manager)

	mux := http.NewServeMux()
	mux.HandleFunc("/v1/executions", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			handler.CreateExecution(w, r)
		}
	})
	mux.HandleFunc("/v1/executions/", func(w http.ResponseWriter, r *http.Request) {
		path := strings.TrimPrefix(r.URL.Path, "/v1/executions/")
		parts := strings.Split(path, "/")
		id := parts[0]

		if len(parts) == 1 && r.Method == http.MethodGet {
			handler.GetExecution(w, r, id)
		} else if len(parts) == 2 && parts[1] == "cancel" && r.Method == http.MethodPost {
			handler.CancelExecution(w, r, id)
		} else {
			http.Error(w, "Not Found", http.StatusNotFound)
		}
	})

	server := httptest.NewServer(mux)
	defer server.Close()

	t.Run("SuccessLifecycle", func(t *testing.T) {
		req := &pb.ExecuteRequest{
			Language: "python",
			Files:    []*pb.SourceFile{{Path: "main.py", Contents: []byte("print('Done')")}},
		}
		reqBody, _ := json.Marshal(req)

		resp, err := http.Post(server.URL+"/v1/executions", "application/json", bytes.NewBuffer(reqBody))
		if err != nil || resp.StatusCode != http.StatusAccepted {
			t.Fatalf("POST failed with status: %d", resp.StatusCode)
		}
		var postRes map[string]string
		json.NewDecoder(resp.Body).Decode(&postRes)
		resp.Body.Close()
		jobID := postRes["id"]

		timeout := time.After(3 * time.Second)
		for {
			select {
			case <-timeout:
				t.Fatalf("Timeout waiting for COMPLETED state.")
			default:
				getResp, _ := http.Get(server.URL + "/v1/executions/" + jobID)
				var getRes map[string]interface{}
				json.NewDecoder(getResp.Body).Decode(&getRes)
				getResp.Body.Close()

				if getRes["state"].(string) == string(api.StateCompleted) {
					return
				}
				time.Sleep(50 * time.Millisecond)
			}
		}
	})

	t.Run("CancellationLifecycle", func(t *testing.T) {
		req := &pb.ExecuteRequest{
			Language: "python",
			Files: []*pb.SourceFile{{
				Path: "main.py",
				// Make it noisy so Rust interacts with the gRPC stream and notices the disconnect!
				Contents: []byte("import time, sys\nwhile True:\n  time.sleep(0.1)"),
			}},
		}
		reqBody, _ := json.Marshal(req)

		resp, _ := http.Post(server.URL+"/v1/executions", "application/json", bytes.NewBuffer(reqBody))
		var postRes map[string]string
		json.NewDecoder(resp.Body).Decode(&postRes)
		resp.Body.Close()
		jobID := postRes["id"]

		// Wait briefly to ensure the background goroutine transitions it to RUNNING
		time.Sleep(200 * time.Millisecond)

		cancelReq, _ := http.NewRequest(http.MethodPost, server.URL+"/v1/executions/"+jobID+"/cancel", nil)
		cancelResp, err := http.DefaultClient.Do(cancelReq)
		if err != nil {
			t.Fatalf("Cancel request failed: %v", err)
		}
		if cancelResp.StatusCode != http.StatusNoContent {
			t.Fatalf("Cancel endpoint failed! Expected 204, got %d. Check routing.", cancelResp.StatusCode)
		}
		cancelResp.Body.Close()

		time.Sleep(100 * time.Millisecond) // Allow Store update to process

		getResp, _ := http.Get(server.URL + "/v1/executions/" + jobID)
		var getRes map[string]interface{}
		json.NewDecoder(getResp.Body).Decode(&getRes)
		getResp.Body.Close()

		if getRes["state"].(string) != string(api.StateCancelled) {
			t.Errorf("Expected state CANCELLED, got %s", getRes["state"])
		}

		// Give the Rust worker a split second to catch the gRPC disconnect on its next write
		// and gracefully kill the process group before the test runner drops the hammer!
		time.Sleep(300 * time.Millisecond)
	})
}

func TestExecutionLifecycle_WebSocketFlow(t *testing.T) {
	addr := startWorkerd(t)
	grpcClient := clientFor(t, addr)

	store := api.NewInMemoryStore()
	manager := api.NewExecutionManager(store)
	handler := api.NewHandler(grpcClient, manager)

	mux := http.NewServeMux()
	mux.HandleFunc("/v1/executions", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			handler.CreateExecution(w, r)
		}
	})
	mux.HandleFunc("/v1/executions/", func(w http.ResponseWriter, r *http.Request) {
		path := strings.TrimPrefix(r.URL.Path, "/v1/executions/")
		parts := strings.Split(path, "/")
		id := parts[0]

		if len(parts) == 1 && r.Method == http.MethodGet {
			handler.GetExecution(w, r, id)
		} else if len(parts) == 2 && parts[1] == "stream" && r.Method == http.MethodGet {
			handler.StreamExecution(w, r, id)
		}
	})

	server := httptest.NewServer(mux)
	defer server.Close()

	// Use the exact request formation that passes in TestGoAPIConcurrentWebSockets
	req := &pb.ExecuteRequest{
		Language: "python",
		Files:    []*pb.SourceFile{{Path: "main.py", Contents: []byte("import time, sys\nprint('Done')\nsys.stdout.flush()\ntime.sleep(0.5)")}},
	}
	reqBody, _ := json.Marshal(req)

	resp, _ := http.Post(server.URL+"/v1/executions", "application/json", bytes.NewBuffer(reqBody))
	var postRes map[string]string
	json.NewDecoder(resp.Body).Decode(&postRes)
	resp.Body.Close()
	jobID := postRes["id"]

	wsURL := "ws" + strings.TrimPrefix(server.URL, "http") + "/v1/executions/" + jobID + "/stream"
	conn, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	if err != nil {
		t.Fatalf("WS connect failed: %v", err)
	}
	defer conn.Close()

	var hasStarted, hasFinished bool
	var stdout []byte

	for {
		conn.SetReadDeadline(time.Now().Add(2 * time.Second))
		_, msg, err := conn.ReadMessage()
		if err != nil {
			break
		}

		var event map[string]interface{}
		if err := json.Unmarshal(msg, &event); err != nil {
			continue
		}

		var evtType float64
		if val, exists := event["type"]; !exists || val == nil {
			evtType = 0
		} else if v, ok := val.(float64); ok {
			evtType = v
		} else {
			continue
		}

		if evtType == 0 {
			hasStarted = true
		}
		if evtType == 1 {
			if dataStr, ok := event["data"].(string); ok {
				decoded, _ := base64.StdEncoding.DecodeString(dataStr)
				stdout = append(stdout, decoded...)
			}
		}
		if evtType == 3 {
			hasFinished = true
		}
	}

	if !hasStarted {
		t.Errorf("Expected STARTED event in WebSocket stream")
	}
	if !hasFinished {
		t.Errorf("Expected FINISHED event in WebSocket stream")
	}
	if strings.TrimSpace(string(stdout)) != "Done" {
		t.Errorf("Expected stdout 'Done', got %q", string(stdout))
	}
}
