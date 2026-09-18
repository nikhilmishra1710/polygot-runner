// Package tests contains end-to-end integration tests:
//
//	Go gRPC Client → Rust workerd (TCP 50051) → Worker → ExecutionEngine → Sandbox
//
// Nothing is mocked. Tests that require the sandboxed worker need a
// built workerd binary and root (namespaces/cgroups); they skip otherwise.
// Pure validation tests run directly against the Go API layer.
package tests

import (
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

// startWorkerd spawns the real workerd and waits for it to bind to TCP 50051.
func startWorkerd(t *testing.T) string {
	t.Helper()
	if os.Geteuid() != 0 {
		t.Skip("requires root: sandbox uses namespaces and cgroups")
	}
	bin := workerdBin(t)
	targetAddr := "127.0.0.1:50051"

	cmd := exec.Command(bin)
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

	// 2. Start the Go API layer in a test HTTP server
	handler := api.NewHandler(grpcClient)
	mux := http.NewServeMux()
	mux.HandleFunc("/ws/v1/execute", handler.StreamHandler)
	server := httptest.NewServer(mux)
	defer server.Close()

	// Convert http:// to ws://
	wsURL := "ws" + strings.TrimPrefix(server.URL, "http") + "/ws/v1/execute"

	var wg sync.WaitGroup
	startTime := time.Now()

	// 3. Define the concurrent worker function
	runClient := func(identity string) {
		defer wg.Done()

		// Dial the Go WebSocket endpoint
		conn, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
		if err != nil {
			t.Errorf("Client %s failed to connect: %v", identity, err)
			return
		}
		defer conn.Close()

		// Python script that takes ~1.5s to run
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

		// Send request over WebSocket
		if err := conn.WriteJSON(req); err != nil {
			t.Errorf("Client %s failed to send: %v", identity, err)
			return
		}

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

			// We use a generic map to parse the JSON for flexibility in the test
			var event map[string]interface{}
			if err := json.Unmarshal(msg, &event); err != nil {
				continue
			}

			// Check for STDOUT events (Type 1)
			if evtType, ok := event["type"].(float64); ok && evtType == 1 {
				// The base64 output comes in the "data" field
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

	// 4. Fire off 3 concurrent WebSocket clients
	wg.Add(3)
	go runClient("A")
	go runClient("B")
	go runClient("C")

	wg.Wait()
	elapsed := time.Since(startTime)

	// 5. Assert Parallelism
	// 3 jobs taking 1.5s each would take 4.5s sequentially.
	// If it takes < 2.5s, Go is correctly handling concurrent WebSocket upgrades.
	if elapsed > 2500*time.Millisecond {
		t.Errorf("Go API serialized the requests! Total time: %v", elapsed)
	}
}
