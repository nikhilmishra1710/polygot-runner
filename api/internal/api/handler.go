package api

import (
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"log"
	"net/http"
	"time"
	"uuid"

	pb "runtime-platform/api/gen/execution/v1"

	"github.com/gorilla/websocket"
	"golang.org/x/crypto/bcrypt"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

type Handler struct {
	workerClient pb.ExecutionServiceClient
	manager      *ExecutionManager
	userStore    UserStore
	sessionStore SessionStore
}

func NewHandler(wc pb.ExecutionServiceClient, manager *ExecutionManager, us UserStore, ss SessionStore) *Handler {
	return &Handler{
		workerClient: wc,
		manager:      manager,
		userStore:    us,
		sessionStore: ss,
	}
}

// POST /v1/executions
func (h *Handler) CreateExecution(w http.ResponseWriter, r *http.Request) {
	user, ok := r.Context().Value(UserContextKey).(*User)
	if !ok {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	var req pb.ExecuteRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request JSON: "+err.Error(), http.StatusBadRequest)
		return
	}

	jobID, execCtx := h.manager.Start(context.Background(), user.ID, &req)

	go func() {
		defer func() {
			// Wake up any waiting WebSockets one final time before closing
			h.manager.mu.Lock()
			for _, ch := range h.manager.listeners[jobID] {
				select {
				case ch <- struct{}{}:
				default:
				}
			}
			h.manager.mu.Unlock()

			h.manager.Cleanup(jobID)
		}()

		// The Store safely ignores this if the job was already CANCELLED
		h.manager.Store.UpdateState(context.Background(), jobID, StateRunning, nil)

		stream, err := h.workerClient.Execute(execCtx, &req)
		if err != nil {
			h.manager.Store.UpdateState(context.Background(), jobID, StateFailed, nil)
			return
		}

		for {
			event, err := stream.Recv()
			if err != nil {
				// Context canceled or stream broken
				h.manager.Store.UpdateState(context.Background(), jobID, StateFailed, nil)
				break
			}

			if event.Type == pb.ExecutionEvent_FINISHED {
				h.manager.Store.UpdateState(context.Background(), jobID, StateCompleted, event.FinalResult)
			}

			h.manager.RouteEvent(jobID, event)
		}
	}()

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	json.NewEncoder(w).Encode(map[string]string{"id": jobID})
}

// GET /v1/executions/{id}
func (h *Handler) GetExecution(w http.ResponseWriter, r *http.Request, id string) {
	user, ok := r.Context().Value(UserContextKey).(*User)
	if !ok {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}
	exec, err := h.manager.Store.Get(r.Context(), user.ID, id)
	if err != nil {
		http.Error(w, "Execution not found", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"id":         exec.ID,
		"state":      exec.State,
		"created_at": exec.CreatedAt,
		"result":     exec.Result,
	})
}

// GET /v1/executions/{id}/stream (Upgrades to WebSocket)
func (h *Handler) StreamExecution(w http.ResponseWriter, r *http.Request, id string) {
	user, ok := r.Context().Value(UserContextKey).(*User)
	if !ok {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	// Register this specific client connection for wakeups
	wakeup := h.manager.Subscribe(id)
	defer h.manager.Unsubscribe(id, wakeup)

	cursor := 0

	for {
		// 1. Fetch the unified timeline from the store
		events, err := h.manager.Store.GetEvents(r.Context(), id)
		if err != nil {
			conn.WriteJSON(map[string]string{"error": "Execution not found"})
			return
		}

		// 2. Play back any events we haven't sent to this client yet
		for i := cursor; i < len(events); i++ {
			if err := conn.WriteJSON(events[i]); err != nil {
				return // Client disconnected
			}
			cursor++
		}

		// 3. Check if the job is finished
		exec, _ := h.manager.Store.Get(r.Context(), user.ID, id)
		if exec.State != StateRunning && exec.State != StateCreated {
			break // Exit the loop gracefully
		}

		// 4. Block until the manager signals new data, or the client disconnects
		<-wakeup
	}

	conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
}

// POST /v1/executions/{id}/cancel
func (h *Handler) CancelExecution(w http.ResponseWriter, r *http.Request, id string) {
	// 1. Explicitly lock the state as CANCELLED first!
	// Our strict Store rules will now prevent the dying goroutine from changing this to FAILED.
	h.manager.Store.UpdateState(context.Background(), id, StateCancelled, nil)

	// 2. Kill the Rust gRPC process
	h.manager.Cancel(id)

	w.WriteHeader(http.StatusNoContent)
}

// POST /v1/executions/{id}/cancel
func (h *Handler) VerifyCookie(w http.ResponseWriter, r *http.Request) {
	user, ok := r.Context().Value(UserContextKey).(*User)
	if !ok {
		log.Println("No user context object")
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	w.WriteHeader(http.StatusOK)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"id": user.ID, "username": user.UserName})
}

func (h *Handler) HandleLogin(w http.ResponseWriter, r *http.Request) {

	var req struct {
		UserID   string `json:"username"`
		Password string `json:"password"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Bad Request", http.StatusBadRequest)
		return
	}
	userID := req.UserID

	user, err := h.userStore.GetByUserName(r.Context(), userID)

	if err != nil {
		http.Error(w, "user not found", http.StatusNotFound)
		return
	}

	err = bcrypt.CompareHashAndPassword([]byte(user.HashedPassword), []byte(req.Password))
	if err != nil {
		http.Error(w, "invalid username or password", http.StatusUnauthorized)
		return
	}

	bytes := make([]byte, 32)
	_, err = rand.Read(bytes)
	if err != nil {
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}

	token := base64.URLEncoding.EncodeToString(bytes)
	sum := sha256.Sum256([]byte(token))
	hash := hex.EncodeToString(sum[:])

	session := Session{
		ID:                 token,
		UserID:             user.ID,
		HashedSessionToken: hash,
		CreatedAt:          time.Now().UTC(),
		ExpiresAt:          time.Now().UTC().AddDate(0, 0, 1),
	}

	if err := h.sessionStore.Create(r.Context(), &session); err != nil {
		http.Error(w, "Failed to create session", http.StatusInternalServerError)
		return
	}

	cookie := http.Cookie{
		Name:     "session_token",
		Value:    token,
		Path:     "/",
		MaxAge:   24 * 60 * 60,
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteLaxMode,
	}

	http.SetCookie(w, &cookie)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "success"})
}

func (h *Handler) HandleSignup(w http.ResponseWriter, r *http.Request) {

	var req struct {
		UserName string `json:"username"`
		Password string `json:"password"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Bad Request", http.StatusBadRequest)
		return
	}

	user, _ := h.userStore.GetByUserName(r.Context(), req.UserName)

	if user != nil {
		http.Error(w, "user already Exists", http.StatusConflict)
		return
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
	if err != nil {
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}
	hashStr := string(hash)
	userStruct := User{
		ID:             uuid.New().String(),
		UserName:       req.UserName,
		HashedPassword: hashStr,
		CreatedAt:      time.Now().UTC(),
		ModifiedAt:     time.Now().UTC(),
	}

	err = h.userStore.Create(r.Context(), &userStruct)

	if err != nil {
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "success"})
}
