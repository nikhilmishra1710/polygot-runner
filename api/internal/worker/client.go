package worker

import (
	"context"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"net"
	"time"
)

// WorkerClient defines the interface for communicating with the worker
type WorkerClient interface {
	Execute(ctx context.Context, req ExecutionRequest) (ExecutionResult, error)
	Close() error
}

// ExecutionRequest represents a request to execute code
type ExecutionRequest struct {
	Language string
	Files    []File
	Stdin    []byte
	Limits   ResourceLimits
}

// File represents a file in the execution request
type File struct {
	Path     string
	Contents []byte
}

// ExecutionResult represents the result of execution
type ExecutionResult struct {
	ID       string
	Status   string
	Stdout   []byte
	Stderr   []byte
	ExitCode *int32 // nil if not available (e.g., timed out)
}

// ResourceLimits mirrors the Rust default limits
type ResourceLimits struct {
	WallTime     Duration `json:"wall_time"`
	CPUTime      Duration `json:"cpu_time"`
	MaxOpenFiles uint64   `json:"max_open_files"`
	MaxFileSize  uint64   `json:"max_file_size"`
	MemoryBytes  uint64   `json:"memory_bytes"`
	PIDsMax      uint32   `json:"pids_max"`
}

// Duration mirrors time.Duration as serialized by serde_json
type Duration struct {
	Secs  int64 `json:"secs"`
	Nanos int32 `json:"nanos"`
}

// UnixWorkerClient implements WorkerClient using Unix domain sockets.
type UnixWorkerClient struct {
	addr   string
	closed bool
}

// NewUnixClient creates a worker client for the given unix socket path.
// The connection is established lazily on the first Execute call, so the
// API can start even when workerd is not yet running.
func NewUnixClient(socketPath string) (*UnixWorkerClient, error) {
	return &UnixWorkerClient{addr: socketPath}, nil
}

// Execute sends an execution request to the worker via Unix socket.
// Each call opens a fresh connection; workerd handles one connection at
// a time and loop-serves it, which keeps this client simple.
func (c *UnixWorkerClient) Execute(ctx context.Context, req ExecutionRequest) (ExecutionResult, error) {
	if c.closed {
		return ExecutionResult{}, ErrClientClosed
	}

	conn, err := net.Dial("unix", c.addr)
	if err != nil {
		return ExecutionResult{}, ErrWorkerUnavailable{Err: err}
	}
	defer conn.Close()

	// Set deadlines based on context timeout
	if deadline, ok := ctx.Deadline(); ok {
		timeout := time.Until(deadline)
		if timeout > 0 {
			conn.SetDeadline(time.Now().Add(timeout))
		} else {
			// deadline already passed
			return ExecutionResult{}, ctx.Err()
		}
	}

	// Build worker request: externally tagged enum
	workerReq := map[string]interface{}{
		"Execute": toWorkerJob(req),
	}
	reqBytes, err := json.Marshal(workerReq)
	if err != nil {
		return ExecutionResult{}, err
	}

	// Write length-prefixed frame (4-byte BE length + payload)
	if err := writeFrame(conn, reqBytes); err != nil {
		return ExecutionResult{}, err
	}

	// Read frame
	respBytes, err := readFrame(conn)
	if err != nil {
		return ExecutionResult{}, err
	}

	// Parse worker response: externally tagged enum
	var respMap map[string]json.RawMessage
	if err := json.Unmarshal(respBytes, &respMap); err != nil {
		return ExecutionResult{}, err
	}
	if val, ok := respMap["Result"]; ok {
		var jobResult JobResult
		if err := json.Unmarshal(val, &jobResult); err != nil {
			return ExecutionResult{}, err
		}
		return toExecutionResult(jobResult), nil
	}
	if val, ok := respMap["Error"]; ok {
		var errStr string
		if err := json.Unmarshal(val, &errStr); err != nil {
			return ExecutionResult{}, err
		}
		return ExecutionResult{}, ErrWorkerError{errStr}
	}
	return ExecutionResult{}, ErrMalformedResponse
}

// Close marks the worker client as closed.
func (c *UnixWorkerClient) Close() error {
	c.closed = true
	return nil
}

// Helper functions

var (
	ErrClientClosed      = workerError("client is closed")
	ErrMalformedResponse = workerError("malformed worker response")
)

type workerError string

func (e workerError) Error() string { return string(e) }

// ErrWorkerUnavailable indicates workerd could not be reached.
type ErrWorkerUnavailable struct {
	Err error
}

func (e ErrWorkerUnavailable) Error() string { return "worker unavailable: " + e.Err.Error() }
func (e ErrWorkerUnavailable) Unwrap() error { return e.Err }

type ErrWorkerError struct {
	Msg string
}

func (e ErrWorkerError) Error() string { return "worker error: " + e.Msg }

// ByteList encodes []byte as a JSON array of integers, matching serde's
// representation of Vec<u8> (Go's encoding/json would emit base64 instead).
type ByteList []byte

func (b ByteList) MarshalJSON() ([]byte, error) {
	nums := make([]int, len(b))
	for i, c := range b {
		nums[i] = int(c)
	}
	return json.Marshal(nums)
}

func (b *ByteList) UnmarshalJSON(data []byte) error {
	var nums []int
	if err := json.Unmarshal(data, &nums); err != nil {
		return err
	}
	out := make([]byte, len(nums))
	for i, n := range nums {
		if v := int64(n); v < 0 || v > 255 {
			return errInvalidByte{n}
		}
		out[i] = byte(n)
	}
	*b = out
	return nil
}

type errInvalidByte struct{ v int }

func (e errInvalidByte) Error() string {
	return fmt.Sprintf("byte sequence value out of range: %d", e.v)
}

// toWorkerJob converts ExecutionRequest to ExecutionJob with a generated ID
func toWorkerJob(req ExecutionRequest) interface{} {
	return map[string]interface{}{
		"id":      fmt.Sprintf("go-%d", time.Now().UnixNano()),
		"request": toWorkerExecutionRequest(req),
	}
}

// workerLanguage maps the public lowercase language name to the worker's
// serde enum variant name.
func workerLanguage(lang string) string {
	switch lang {
	case "python":
		return "Python"
	case "cpp":
		return "Cpp"
	}
	// Unrecognized languages are rejected by API validation before this point.
	return lang
}

// toWorkerExecutionRequest converts ExecutionRequest to worker ExecutionRequest
func toWorkerExecutionRequest(req ExecutionRequest) interface{} {
	return map[string]interface{}{
		"language": workerLanguage(req.Language),
		"files":    toWorkerFiles(req.Files),
		"stdin":    ByteList(req.Stdin),
		"limits":   req.Limits,
	}
}

func toWorkerFiles(files []File) []interface{} {
	res := make([]interface{}, len(files))
	for i, f := range files {
		res[i] = map[string]interface{}{
			"path":     f.Path,
			"contents": ByteList(f.Contents),
		}
	}
	return res
}

// toExecutionResult converts Worker JobResult to ExecutionResult
func toExecutionResult(jr JobResult) ExecutionResult {
	status := mapTerminationToStatus(jr.Report.Termination)
	return ExecutionResult{
		ID:       jr.ID,
		Status:   status,
		Stdout:   []byte(jr.Report.Output.Stdout),
		Stderr:   []byte(jr.Report.Output.Stderr),
		ExitCode: exitCodeFromTermination(jr.Report.Termination),
	}
}

// mapTerminationToStatus converts TerminationReason to public ExecutionStatus string
func mapTerminationToStatus(tr TerminationReason) string {
	// Check which field is set
	if tr.ExitCode != nil {
		if *tr.ExitCode == 0 {
			return "SUCCESS"
		}
		return "RUNTIME_ERROR"
	}
	if tr.Signal != nil {
		return "RUNTIME_ERROR"
	}
	if tr.WallTimeout != nil && *tr.WallTimeout {
		return "TIME_LIMIT_EXCEEDED"
	}
	if tr.CpuLimit != nil && *tr.CpuLimit {
		return "RESOURCE_LIMIT_EXCEEDED"
	}
	if tr.MemoryLimit != nil && *tr.MemoryLimit {
		return "RESOURCE_LIMIT_EXCEEDED"
	}
	if tr.OomKilled != nil && *tr.OomKilled {
		return "RESOURCE_LIMIT_EXCEEDED"
	}
	if tr.SeccompViolation != nil && *tr.SeccompViolation {
		return "RUNTIME_ERROR"
	}
	// fallback
	return "RUNTIME_ERROR"
}

// exitCodeFromTermination returns exit code if available
func exitCodeFromTermination(tr TerminationReason) *int32 {
	if tr.ExitCode != nil {
		return tr.ExitCode
	}
	return nil
}

// --- Definitions for unmarshalling worker response ---

// JobResult mirrors the Rust struct
type JobResult struct {
	ID     string          `json:"id"`
	Report ExecutionReport `json:"report"`
}

// ExecutionReport mirrors the Rust struct
type ExecutionReport struct {
	Output      Output            `json:"output"`
	Termination TerminationReason `json:"termination"`
	Metrics     ExecutionMetrics  `json:"metrics"`
}

// Output mirrors the Rust struct (stdout/stderr are JSON integer arrays).
type Output struct {
	Stdout ByteList `json:"stdout"`
	Stderr ByteList `json:"stderr"`
}

// ExecutionMetrics mirrors the Rust struct
type ExecutionMetrics struct {
	WallTime    Duration `json:"wall_time"`
	CPUTime     Duration `json:"cpu_time"`
	MaxRSSBytes uint64   `json:"max_rss_bytes"`
	PeakPIDs    uint64   `json:"peak_pids"`
}

// TerminationReason mirrors the Rust enum via explicit fields
type TerminationReason struct {
	ExitCode         *int32 `json:"ExitCode,omitempty"`
	Signal           *int32 `json:"Signal,omitempty"`
	WallTimeout      *bool  `json:"WallTimeout,omitempty"`
	CpuLimit         *bool  `json:"CpuLimit,omitempty"`
	MemoryLimit      *bool  `json:"MemoryLimit,omitempty"`
	OomKilled        *bool  `json:"OomKilled,omitempty"`
	SeccompViolation *bool  `json:"SeccompViolation,omitempty"`
}

// writeFrame writes a length-prefixed frame: 4-byte BE length + payload
func writeFrame(w net.Conn, b []byte) error {
	if len(b) > 0xFFFFFFFF {
		return frameTooLargeError{len(b)}
	}
	var lenBuf [4]byte
	binary.BigEndian.PutUint32(lenBuf[:], uint32(len(b)))
	if _, err := w.Write(lenBuf[:]); err != nil {
		return err
	}
	if _, err := w.Write(b); err != nil {
		return err
	}
	return nil
}

// readFrame reads a length-prefixed frame: reads 4-byte BE length then payload
func readFrame(r net.Conn) ([]byte, error) {
	var lenBuf [4]byte
	if _, err := r.Read(lenBuf[:]); err != nil {
		return nil, err
	}
	length := binary.BigEndian.Uint32(lenBuf[:])
	if length > 0xFFFFFFFF {
		return nil, frameTooLargeError{int(length)}
	}
	payload := make([]byte, length)
	if _, err := r.Read(payload); err != nil {
		return nil, err
	}
	return payload, nil
}

type frameTooLargeError struct {
	size int
}

func (e frameTooLargeError) Error() string {
	return fmt.Sprintf("frame too large: %d bytes", e.size)
}
