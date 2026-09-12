package api

import (
	"fmt"
	"testing"

	pb "runtime-platform/api/gen/execution/v1"
)

func TestValidateExecutionRequest(t *testing.T) {
	tests := []struct {
		name    string
		req     pb.ExecuteRequest
		wantErr bool
	}{
		{
			name: "valid python",
			req: pb.ExecuteRequest{
				Language: "python",
				Files: []*pb.SourceFile{
					{Path: "main.py", Contents: []byte(`print("hi")`)},
				},
			},
			wantErr: false,
		},
		{
			name: "valid cpp",
			req: pb.ExecuteRequest{
				Language: "cpp",
				Files: []*pb.SourceFile{
					{Path: "main.cpp", Contents: []byte(`int main(){return 0;}`)},
				},
			},
			wantErr: false,
		},
		{
			name: "missing language",
			req: pb.ExecuteRequest{
				Files: []*pb.SourceFile{{Path: "x", Contents: []byte{}}},
			},
			wantErr: true,
		},
		{
			name: "unsupported language",
			req: pb.ExecuteRequest{
				Language: "java",
				Files:    []*pb.SourceFile{{Path: "x", Contents: []byte{}}},
			},
			wantErr: true,
		},
		{
			name: "no files",
			req: pb.ExecuteRequest{
				Language: "python",
			},
			wantErr: true,
		},
		{
			name: "too many files",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    makeFiles(33),
			},
			wantErr: true,
		},
		{
			name: "empty path",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    []*pb.SourceFile{{Path: "", Contents: []byte{}}},
			},
			wantErr: true,
		},
		{
			name: "absolute path",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    []*pb.SourceFile{{Path: "/etc/passwd", Contents: []byte{}}},
			},
			wantErr: true,
		},
		{
			name: "path with ..",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    []*pb.SourceFile{{Path: "foo/../bar.py", Contents: []byte{}}},
			},
			wantErr: true,
		},
		{
			name: "path not cleaned",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    []*pb.SourceFile{{Path: "./foo/bar.py", Contents: []byte{}}},
			},
			wantErr: true,
		},
		{
			name: "double slash",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    []*pb.SourceFile{{Path: "foo//bar.py", Contents: []byte{}}},
			},
			wantErr: true,
		},
		{
			name: "file too large",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    []*pb.SourceFile{{Path: "big.py", Contents: bytesRepeat('a', maxSourcePerFile+1)}},
			},
			wantErr: true,
		},
		{
			name: "total source too large",
			req: pb.ExecuteRequest{
				Language: "python",
				Files: []*pb.SourceFile{
					{Path: "a.py", Contents: bytesRepeat('a', maxTotalSource/2+1)},
					{Path: "b.py", Contents: bytesRepeat('a', maxTotalSource/2+1)},
				},
			},
			wantErr: true,
		},
		{
			name: "stdin too large",
			req: pb.ExecuteRequest{
				Language: "python",
				Files:    []*pb.SourceFile{{Path: "x.py", Contents: []byte{}}},
				Stdin:    bytesRepeat('b', maxStdinSize+1),
			},
			wantErr: true,
		},
	}
	for i := range tests {
		tt := &tests[i]
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateExecutionRequest(&tt.req)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateExecutionRequest() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

// helpers
func makeFiles(n int) []*pb.SourceFile {
	files := make([]*pb.SourceFile, n)
	for i := 0; i < n; i++ {
		files[i] = &pb.SourceFile{
			Path:     fmt.Sprintf("file%d.py", i),
			Contents: []byte{byte(i)},
		}
	}
	return files
}
func bytesRepeat(b byte, n int) []byte {
	out := make([]byte, n)
	for i := range out {
		out[i] = b
	}
	return out
}
