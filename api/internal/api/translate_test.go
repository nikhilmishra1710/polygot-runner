package api

import (
	"bytes"
	"testing"

	"runtime-platform/api/internal/worker"

	pb "runtime-platform/api/gen/execution/v1"
)

func TestProtoRequestToWorker(t *testing.T) {
	req := &pb.ExecuteRequest{
		Language: "python",
		Files: []*pb.SourceFile{
			{Path: "a.py", Contents: []byte("print(1)")},
			{Path: "sub/b.cpp", Contents: []byte("int main(){}")},
		},
		Stdin: []byte("input"),
		// Limits field not present in proto; we expect defaults.
	}
	wreq := ProtoRequestToWorker(req)
	if wreq.Language != "python" {
		t.Errorf("Language = %q, want python", wreq.Language)
	}
	if len(wreq.Files) != 2 {
		t.Errorf("Files len = %d, want 2", len(wreq.Files))
	}
	if string(wreq.Files[0].Contents) != "print(1)" {
		t.Errorf("File0 contents mismatch")
	}
	if wreq.Files[0].Path != "a.py" {
		t.Errorf("File0 path mismatch")
	}
	if wreq.Files[1].Path != "sub/b.cpp" {
		t.Errorf("File1 path mismatch")
	}
	if string(wreq.Stdin) != "input" {
		t.Errorf("Stdin mismatch")
	}
	// check limits are defaults
	if wreq.Limits.WallTime.Secs != 10 || wreq.Limits.WallTime.Nanos != 0 {
		t.Errorf("WallTime mismatch: got %+v, want {Secs:10 Nanos:0}", wreq.Limits.WallTime)
	}
	if wreq.Limits.CPUTime.Secs != 10 || wreq.Limits.CPUTime.Nanos != 0 {
		t.Errorf("CPUTime mismatch")
	}
	if wreq.Limits.MaxOpenFiles != 128 {
		t.Errorf("MaxOpenFiles mismatch")
	}
	if wreq.Limits.MaxFileSize != 10*1024*1024 {
		t.Errorf("MaxFileSize mismatch")
	}
	if wreq.Limits.MemoryBytes != 256*1024*1024 {
		t.Errorf("MemoryBytes mismatch")
	}
	if wreq.Limits.PIDsMax != 64 {
		t.Errorf("PIDsMax mismatch")
	}
}

func TestWorkerResultToProtoResponse(t *testing.T) {
	// Success case
	res := worker.ExecutionResult{
		ID:       "abc",
		Status:   "SUCCESS",
		Stdout:   []byte("out\n"),
		Stderr:   []byte("err"),
		ExitCode: ptrInt32(0),
	}
	resp := WorkerResultToProtoResponse("abc", res)
	if resp.GetId() != "abc" {
		t.Errorf("ID mismatch")
	}
	if resp.GetStatus() != pb.ExecutionStatus_SUCCESS {
		t.Errorf("Status mismatch")
	}
	if !bytes.Equal(resp.GetStdout(), res.Stdout) {
		t.Errorf("Stdout mismatch")
	}
	if !bytes.Equal(resp.GetStderr(), res.Stderr) {
		t.Errorf("Stderr mismatch")
	}
	if resp.ExitCode == nil {
		t.Errorf("ExitCode should be set")
	}
	if *resp.ExitCode != 0 {
		t.Errorf("ExitCode mismatch")
	}
	// Runtime error with exit code 1
	res2 := worker.ExecutionResult{
		ID:       "def",
		Status:   "RUNTIME_ERROR",
		Stdout:   []byte{},
		Stderr:   []byte("panic"),
		ExitCode: ptrInt32(1),
	}
	resp2 := WorkerResultToProtoResponse("def", res2)
	if resp2.GetStatus() != pb.ExecutionStatus_RUNTIME_ERROR {
		t.Errorf("Status mismatch for runtime error")
	}
	if resp2.ExitCode == nil {
		t.Errorf("ExitCode should be set")
	}
	if *resp2.ExitCode != 1 {
		t.Errorf("ExitCode mismatch")
	}
	// Time limit exceeded
	res3 := worker.ExecutionResult{
		ID:       "ghi",
		Status:   "TIME_LIMIT_EXCEEDED",
		Stdout:   []byte{},
		Stderr:   []byte{},
		ExitCode: nil,
	}
	resp3 := WorkerResultToProtoResponse("ghi", res3)
	if resp3.GetStatus() != pb.ExecutionStatus_TIME_LIMIT_EXCEEDED {
		t.Errorf("Status mismatch for timeout")
	}
	if resp3.ExitCode != nil {
		t.Errorf("ExitCode should be nil for timeout")
	}
}

// helpers
func ptrInt32(v int32) *int32 { return &v }
