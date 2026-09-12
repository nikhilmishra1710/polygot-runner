// Package tests contains end-to-end integration tests:
//
//	gRPC client → Go API → UnixWorkerClient → workerd → Worker →
//	ExecutionEngine → Sandbox → process
//
// Nothing is mocked. Tests that require the sandboxed worker need a
// built workerd binary and root (namespaces/cgroups); they skip otherwise.
// Pure validation tests run everywhere because validation rejects the
// request before the worker is involved.
package tests

import (
	"context"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"

	pb "runtime-platform/api/gen/execution/v1"
	"runtime-platform/api/internal/api"
	"runtime-platform/api/internal/worker"
)

const bufSize = 4 * 1024 * 1024

// clientFor returns a gRPC client wired to a real ExecutionService backed by
// a real UnixWorkerClient connected to socketPath.
func clientFor(t *testing.T, socketPath string) pb.ExecutionServiceClient {
	t.Helper()

	wc, err := worker.NewUnixClient(socketPath)
	if err != nil {
		t.Fatalf("connect to worker socket: %v", err)
	}
	t.Cleanup(func() { wc.Close() })

	server := api.NewServer(wc)

	lis := bufconn.Listen(bufSize)
	grpcServer := grpc.NewServer()
	pb.RegisterExecutionServiceServer(grpcServer, server)
	go func() { _ = grpcServer.Serve(lis) }()
	t.Cleanup(grpcServer.Stop)

	ctx := context.Background()
	conn, err := grpc.DialContext(ctx, "bufnet",
		grpc.WithContextDialer(func(ctx context.Context, _ string) (net.Conn, error) {
			return lis.DialContext(ctx)
		}),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("dial bufconn: %v", err)
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

// startWorkerd spawns the real workerd on a fresh Unix socket.
func startWorkerd(t *testing.T) string {
	t.Helper()
	if os.Geteuid() != 0 {
		t.Skip("requires root: sandbox uses namespaces and cgroups")
	}
	bin := workerdBin(t)

	socketPath := filepath.Join(t.TempDir(), "worker.sock")

	// workerd hardcodes its socket path; run it in an isolated way by
	// overriding via its own CLI if supported, otherwise use the default
	// path only when free. For now workerd uses /tmp/runtime-worker.sock.
	socketPath = "/tmp/runtime-worker.sock"
	_ = os.Remove(socketPath)

	cmd := exec.Command(bin)
	if err := cmd.Start(); err != nil {
		t.Fatalf("start workerd: %v", err)
	}
	t.Cleanup(func() {
		_ = cmd.Process.Kill()
		_ = cmd.Wait()
		_ = os.Remove(socketPath)
	})

	deadline := time.Now().Add(5 * time.Second)
	for {
		if _, err := os.Stat(socketPath); err == nil {
			break
		}
		if time.Now().After(deadline) {
			t.Fatalf("workerd did not create socket %s", socketPath)
		}
		time.Sleep(20 * time.Millisecond)
	}
	return socketPath
}

// --- End-to-end (real workerd) ---

func TestExecutePythonSuccess(t *testing.T) {
	socketPath := startWorkerd(t)
	client := clientFor(t, socketPath)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	resp, err := client.Execute(ctx, &pb.ExecuteRequest{
		Language: "python",
		Files: []*pb.SourceFile{
			{Path: "main.py", Contents: []byte(`print("Hello")`)},
		},
	})
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}
	if string(resp.GetStdout()) != "Hello\n" {
		t.Errorf("stdout = %q, want %q", resp.GetStdout(), "Hello\n")
	}
	if resp.GetStatus() != pb.ExecutionStatus_SUCCESS {
		t.Errorf("status = %v, want SUCCESS", resp.GetStatus())
	}
	if resp.ExitCode == nil || *resp.ExitCode != 0 {
		t.Errorf("exit_code = %+v, want 0", resp.ExitCode)
	}
}

func TestExecutePythonRuntimeError(t *testing.T) {
	socketPath := startWorkerd(t)
	client := clientFor(t, socketPath)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	resp, err := client.Execute(ctx, &pb.ExecuteRequest{
		Language: "python",
		Files: []*pb.SourceFile{
			{Path: "main.py", Contents: []byte(`raise RuntimeError("boom")`)},
		},
	})
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}
	if resp.GetStatus() != pb.ExecutionStatus_RUNTIME_ERROR {
		t.Errorf("status = %v, want RUNTIME_ERROR", resp.GetStatus())
	}
	if resp.ExitCode == nil || *resp.ExitCode == 0 {
		t.Errorf("exit_code = %+v, want non-zero", resp.ExitCode)
	}
	if !strings.Contains(string(resp.GetStderr()), "boom") {
		t.Errorf("stderr = %q, want to contain traceback with boom", resp.GetStderr())
	}
}

// --- Validation: rejected at the API boundary, worker never consulted ---

func TestUnsupportedLanguage(t *testing.T) {
	// Socket won't be dialed because validation fails first.
	client := clientFor(t, "/nonexistent/worker.sock")

	_, err := client.Execute(context.Background(), &pb.ExecuteRequest{
		Language: "java",
		Files:    []*pb.SourceFile{{Path: "Main.java", Contents: []byte("class Main {}")}},
	})
	requireCode(t, err, codes.InvalidArgument)
}

func TestEmptyFiles(t *testing.T) {
	client := clientFor(t, "/nonexistent/worker.sock")

	_, err := client.Execute(context.Background(), &pb.ExecuteRequest{
		Language: "python",
	})
	requireCode(t, err, codes.InvalidArgument)
}

func TestInvalidFilePath(t *testing.T) {
	client := clientFor(t, "/nonexistent/worker.sock")

	_, err := client.Execute(context.Background(), &pb.ExecuteRequest{
		Language: "python",
		Files:    []*pb.SourceFile{{Path: "../evil.py", Contents: []byte("pass")}},
	})
	requireCode(t, err, codes.InvalidArgument)
}

func TestSourceTooLarge(t *testing.T) {
	client := clientFor(t, "/nonexistent/worker.sock")

	big := make([]byte, 256*1024+1)
	_, err := client.Execute(context.Background(), &pb.ExecuteRequest{
		Language: "python",
		Files:    []*pb.SourceFile{{Path: "big.py", Contents: big}},
	})
	requireCode(t, err, codes.InvalidArgument)
}

// --- workerd unavailable ---

func TestWorkerUnavailable(t *testing.T) {
	client := clientFor(t, "/nonexistent/worker.sock")

	_, err := client.Execute(context.Background(), &pb.ExecuteRequest{
		Language: "python",
		Files:    []*pb.SourceFile{{Path: "main.py", Contents: []byte("print(1)")}},
	})
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
