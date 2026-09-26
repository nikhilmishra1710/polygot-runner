package main

import (
	"log"
	"net/http"
	"os"
	"strings"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "runtime-platform/api/gen/execution/v1"
	"runtime-platform/api/internal/api"
)

func main() {
	workerAddr := os.Getenv("WORKER_ADDR")
	if workerAddr == "" {
		workerAddr = "127.0.0.1:50051"
	}

	apiAddr := os.Getenv("API_ADDR")
	if apiAddr == "" {
		apiAddr = "0.0.0.0:8080"
	}

	log.Printf("Connecting to worker at %s...", workerAddr)

	conn, err := grpc.Dial(workerAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect to worker: %v", err)
	}
	defer conn.Close()

	workerClient := pb.NewExecutionServiceClient(conn)
	store := api.NewInMemoryStore()
	manager := api.NewExecutionManager(store)
	userStore := api.NewInMemoryUserStore()
	sessionStore := api.NewInMemorySessionStore()
	handler := api.NewHandler(workerClient, manager, userStore, sessionStore)
	authMiddleware := api.RequireAuth(sessionStore, userStore)

	mux := http.NewServeMux()

	mux.Handle("/v1/verify", authMiddleware(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodGet {
			handler.VerifyCookie(w, r)
		} else {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		}
	})))

	mux.HandleFunc("/v1/login", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			handler.HandleLogin(w, r)
		} else {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		}
	})

	mux.HandleFunc("/v1/signup", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			handler.HandleSignup(w, r)
		} else {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		}
	})

	mux.Handle("/v1/executions", authMiddleware(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			handler.CreateExecution(w, r)
		} else {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		}
	})))

	mux.Handle("/v1/executions/", authMiddleware(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		path := strings.TrimPrefix(r.URL.Path, "/v1/executions/")
		parts := strings.Split(path, "/")
		id := parts[0]

		if id == "" {
			http.Error(w, "Not found", http.StatusNotFound)
			return
		}

		// GET /v1/executions/{id}
		if len(parts) == 1 && r.Method == http.MethodGet {
			handler.GetExecution(w, r, id)
			return
		}

		// GET (WS) /v1/executions/{id}/stream
		if len(parts) == 2 && parts[1] == "stream" && r.Method == http.MethodGet {
			handler.StreamExecution(w, r, id)
			return
		}

		// POST /v1/executions/{id}/cancel
		if len(parts) == 2 && parts[1] == "cancel" && r.Method == http.MethodPost {
			handler.CancelExecution(w, r, id)
			return
		}

		http.Error(w, "Not found", http.StatusNotFound)
	})))

	log.Printf("Starting Go API Gateway on %s...", apiAddr)
	if err := http.ListenAndServe(apiAddr, corsMiddleware(mux)); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}

func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "http://localhost:5173")
		w.Header().Set("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
		w.Header().Set("Access-Control-Allow-Credentials", "true")

		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusOK)
			return
		}
		next.ServeHTTP(w, r)
	})
}
