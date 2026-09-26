# Runtime Platform API

Go API service for executing code in sandboxed environments.

## Structure

```
api/
├── go.mod
├── cmd/
│   └── server/
│       └── main.go
├── internal/
│   ├── api/
│   │   ├── server.go
│   │   └── handlers.go
│   └── worker/
│       └── client.go
└── README.md
```

## Building and Running

```bash
# Build the server
go build -o server ./cmd/server

# Run the server
./server
```

## Running tests

```bash
go test -c ./tests -o tmp/integration.test
sudo ./tmp/integration.test
```

Add optional -test.v for verbose test output

The API will be available at http://localhost:8080

## API Endpoints

### POST /v1/executions

Execute code in a sandboxed environment.

**Request:**

```json
{
  "language": "python",
  "files": [
    {
      "path": "main.py",
      "contents": "print('Hello')"
    }
  ],
  "stdin": ""
}
```

**Response:**

```json
{
  "id": "execution-123",
  "status": "success",
  "stdout": "Hello\n",
  "stderr": "",
  "exit_code": 0
}
```
