package main

import (
	"log"
	"net"
	"os"
	"os/signal"

	"google.golang.org/grpc"

	pb "runtime-platform/api/gen/execution/v1"

	"runtime-platform/api/internal/api"
	"runtime-platform/api/internal/worker"
)

func workerSocketPath() string {
	if p := os.Getenv("WORKER_SOCKET_PATH"); p != "" {
		return p
	}
	return "/tmp/runtime-worker.sock"
}

func main() {
	// Create worker client
	workerClient, err := worker.NewUnixClient(workerSocketPath())
	if err != nil {
		log.Fatalf("Failed to create worker client: %v", err)
	}
	defer workerClient.Close()

	// Create API server
	apiServer := api.NewServer(workerClient)

	// Create gRPC server and register the ExecutionService
	grpcServer := grpc.NewServer()
	pb.RegisterExecutionServiceServer(grpcServer, apiServer)

	// Listen on TCP port 8080
	lis, err := net.Listen("tcp", ":8080")
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	// Start server in goroutine
	go func() {
		log.Println("Starting gRPC API server on :8080")
		if err := grpcServer.Serve(lis); err != nil {
			log.Fatalf("Failed to serve: %v", err)
		}
	}()

	// Wait for interrupt signal to gracefully shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit
	log.Println("Shutting down server...")

	grpcServer.GracefulStop()

	log.Println("Server exited")
}
