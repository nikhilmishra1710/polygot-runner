package main

import (
	"log"
	"net/http"
	"os"
	"os/signal"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "runtime-platform/api/gen/execution/v1"
	"runtime-platform/api/internal/api"
)

func main() {
	// 1. Connect to the Rust gRPC Worker
	conn, err := grpc.Dial("127.0.0.1:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect to Rust worker: %v", err)
	}
	defer conn.Close()

	// Automatically generated gRPC client!
	workerClient := pb.NewExecutionServiceClient(conn)
	handler := api.NewHandler(workerClient)

	// 2. Setup Frontend Routes
	// 2. Setup Frontend Routes
	http.HandleFunc("/ws/v1/execute", handler.StreamHandler) // Live WebSocket
	http.HandleFunc("/v1/execute", handler.SyncHandler)      // Standard REST POST

	// 3. Start the Server
	go func() {
		log.Println("Starting UI Gateway on :8080")
		if err := http.ListenAndServe(":8080", nil); err != nil {
			log.Fatalf("Failed to serve: %v", err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit
	log.Println("Server exited cleanly")
}
