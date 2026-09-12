package api

import (
	"errors"
	"fmt"
	"path/filepath"
	"strings"

	pb "runtime-platform/api/gen/execution/v1"
)

// Validation limits (not security boundaries; worker enforces real limits)
const (
	maxFiles         = 32
	maxSourcePerFile = 256 * 1024  // 256 KiB
	maxTotalSource   = 1024 * 1024 // 1 MiB
	maxStdinSize     = 1024 * 1024 // 1 MiB
)

// validateExecutionRequest validates the proto ExecuteRequest.
// Returns nil if valid, otherwise an error that will be mapped to InvalidArgument.
func ValidateExecutionRequest(req *pb.ExecuteRequest) error {
	if req.GetLanguage() == "" {
		return errors.New("language is required")
	}
	switch req.GetLanguage() {
	case "python", "cpp":
	default:
		return fmt.Errorf("unsupported language: %s", req.GetLanguage())
	}
	files := req.GetFiles()
	if len(files) == 0 {
		return errors.New("at least one file is required")
	}
	if len(files) > maxFiles {
		return fmt.Errorf("too many files: %d (max %d)", len(files), maxFiles)
	}
	var totalSource int64
	for _, f := range files {
		if err := validateFile(f); err != nil {
			return err
		}
		totalSource += int64(len(f.GetContents()))
		if int64(len(f.GetContents())) > maxSourcePerFile {
			return fmt.Errorf("file %q too large: %d bytes (max %d)", f.GetPath(), len(f.GetContents()), maxSourcePerFile)
		}
	}
	if totalSource > maxTotalSource {
		return fmt.Errorf("total source too large: %d bytes (max %d)", totalSource, maxTotalSource)
	}
	if int64(len(req.GetStdin())) > maxStdinSize {
		return fmt.Errorf("stdin too large: %d bytes (max %d)", len(req.GetStdin()), maxStdinSize)
	}
	return nil
}

func validateFile(f *pb.SourceFile) error {
	path := f.GetPath()
	if path == "" {
		return errors.New("file path is required")
	}
	// Must be relative (not absolute)
	if filepath.IsAbs(path) {
		return fmt.Errorf("file path must be relative, got absolute path: %q", path)
	}
	// No .. components
	if strings.Contains(path, "..") {
		return fmt.Errorf("file path must not contain '..': %q", path)
	}
	// Ensure the cleaned path equals the original (no . or extra slashes)
	if cleaned := filepath.Clean(path); cleaned != path {
		// Allow turning "./foo" into "foo"? We'll be strict: require already cleaned.
		return fmt.Errorf("file path must be cleaned: got %q, cleaned %q", path, cleaned)
	}
	// Disallow empty component after splitting (e.g., "foo//bar")
	parts := strings.Split(path, "/")
	for _, p := range parts {
		if p == "" {
			return fmt.Errorf("file path contains empty component: %q", path)
		}
	}
	// Disallow NUL bytes (though Go strings won't have them unless from raw bytes)
	if strings.ContainsRune(path, 0x00) {
		return errors.New("file path must not contain NUL bytes")
	}
	// Contents can be empty? Allow empty file.
	return nil
}
