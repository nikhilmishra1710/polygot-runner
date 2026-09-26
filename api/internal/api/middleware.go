// Typically placed in a file like context.go or middleware.go
package api

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"log"
	"net/http"
	"time"
)

type contextKey string

const UserContextKey = contextKey("user")

func RequireAuth(sessionStore SessionStore, userStore UserStore) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			cookie, err := r.Cookie("session_token")

			if err != nil {
				switch {
				case errors.Is(err, http.ErrNoCookie):
					log.Println("Cookie not found")
					http.Error(w, "cookie not found", http.StatusBadRequest)
				default:
					log.Println(err)
					http.Error(w, "server error", http.StatusInternalServerError)
				}
				return
			}

			sum := sha256.Sum256([]byte(cookie.Value))
			hash := hex.EncodeToString(sum[:])
			
			session, err := sessionStore.Get(r.Context(), hash)
			if err != nil {
				switch err {
				case ErrSessionNotFound:
					http.Error(w, ErrSessionNotFound.Error(), http.StatusUnauthorized)
				}
				return
			}

			expired := session.ExpiresAt.Before(time.Now())
			if expired {
				_ = sessionStore.Delete(r.Context(), hash)
				http.Error(w, ErrSessionExpired.Error(), http.StatusUnauthorized)
				return
			}

			user, err := userStore.GetById(r.Context(), session.UserID)
			if err != nil {
				http.Error(w, ErrUserNotFound.Error(), http.StatusUnauthorized)
				return
			}

			ctx := context.WithValue(r.Context(), UserContextKey, user)

			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}
