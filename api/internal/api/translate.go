package api

import (
	"runtime-platform/api/internal/worker"

	pb "runtime-platform/api/gen/execution/v1"
)

// ProtoRequestToWorker converts proto ExecuteRequest to worker ExecutionRequest.
// Limits are set to the worker's default values (matching Rust's Default).
func ProtoRequestToWorker(req *pb.ExecuteRequest) worker.ExecutionRequest {
	wf := make([]worker.File, len(req.GetFiles()))
	for i, f := range req.GetFiles() {
		wf[i] = worker.File{
			Path:     f.GetPath(),
			Contents: f.GetContents(),
		}
	}
	return worker.ExecutionRequest{
		Language: req.GetLanguage(),
		Files:    wf,
		Stdin:    req.GetStdin(),
		Limits:   defaultResourceLimits(),
	}
}

// WorkerResultToProtoResponse converts worker ExecutionResult to proto ExecuteResponse.
func WorkerResultToProtoResponse(id string, res worker.ExecutionResult) *pb.ExecuteResponse {
	// Map worker status string to proto ExecutionStatus enum
	status := pb.ExecutionStatus_EXECUTION_STATUS_UNSPECIFIED
	switch res.Status {
	case "SUCCESS":
		status = pb.ExecutionStatus_SUCCESS
	case "RUNTIME_ERROR":
		status = pb.ExecutionStatus_RUNTIME_ERROR
	case "TIME_LIMIT_EXCEEDED":
		status = pb.ExecutionStatus_TIME_LIMIT_EXCEEDED
	case "RESOURCE_LIMIT_EXCEEDED":
		status = pb.ExecutionStatus_RESOURCE_LIMIT_EXCEEDED
	default:
		status = pb.ExecutionStatus_EXECUTION_STATUS_UNSPECIFIED
	}
	resp := &pb.ExecuteResponse{
		Id:     id,
		Status: status,
		Stdout: res.Stdout,
		Stderr: res.Stderr,
	}
	if res.ExitCode != nil {
		tmp := *res.ExitCode
		resp.ExitCode = &tmp
	}
	return resp
}

// defaultResourceLimits returns the default limits matching the worker's Default.
func defaultResourceLimits() worker.ResourceLimits {
	return worker.ResourceLimits{
		WallTime:     worker.Duration{Secs: 10, Nanos: 0},
		CPUTime:      worker.Duration{Secs: 10, Nanos: 0},
		MaxOpenFiles: 128,
		MaxFileSize:  10 * 1024 * 1024,  // 10 MiB
		MemoryBytes:  256 * 1024 * 1024, // 256 MB
		PIDsMax:      64,
	}
}
