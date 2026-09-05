package main

import (
	"context"
	"log"
	"net/http"
	"os"
	"os/signal"
	"time"

	"runtime-platform/api/internal/api"
	"runtime-platform/api/internal/worker"
)

func main() {
	// Create worker client
	workerClient, err := worker.NewUnixClient("/tmp/worker.sock")
	if err != nil {
		log.Fatalf("Failed to create worker client: %v", err)
	}
	defer workerClient.Close()

	// Create API server
	apiServer := api.NewServer(workerClient)

	// Create HTTP server
	srv := &http.Server{
		Addr:    ":8080",
		Handler: apiServer.Router(),
	}

	// Start server in goroutine
	go func() {
		log.Println("Starting API server on :8080")
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Failed to start server: %v", err)
		}
	}()

	// Wait for interrupt signal to gracefully shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit
	log.Println("Shutting down server...")

	// Create context with timeout for graceful shutdown
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		log.Fatalf("Server forced to shutdown: %v", err)
	}

	log.Println("Server exited")
}