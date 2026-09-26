// src/pages/Login.tsx
import { useState } from "react";
import { Link } from "react-router-dom";
import { useAuth } from "../context/AuthContext";

export default function Login() {
  const { login } = useAuth();
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);

    // Extract values directly from the DOM to avoid unnecessary React re-renders
    const formData = new FormData(e.currentTarget);
    const username = formData.get("username") as string;
    const password = formData.get("password") as string;

    if (!username || !password) {
      setError("Username and password are required.");
      setIsSubmitting(false);
      return;
    }

    try {
      await login(username, password);
    } catch (err: any) {
      if (err.response?.status === 401) {
        setError("Invalid username or password.");
      } else {
        setError("An unexpected error occurred. Please try again.");
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div style={styles.container}>
      <form onSubmit={handleSubmit} style={styles.form}>
        <h2 style={styles.title}>Welcome Back</h2>

        {error && <div style={styles.error}>{error}</div>}

        <div style={styles.inputGroup}>
          <label htmlFor="username" style={styles.label}>
            Username
          </label>
          <input
            id="username"
            name="username"
            type="text"
            autoComplete="username"
            disabled={isSubmitting}
            style={styles.input}
            required
          />
        </div>

        <div style={styles.inputGroup}>
          <label htmlFor="password" style={styles.label}>
            Password
          </label>
          <input
            id="password"
            name="password"
            type="password"
            autoComplete="current-password"
            disabled={isSubmitting}
            style={styles.input}
            required
          />
        </div>

        <button type="submit" disabled={isSubmitting} style={styles.button}>
          {isSubmitting ? "Signing in..." : "Sign In"}
        </button>

        <div style={styles.footer}>
          Don't have an account?{" "}
          <Link to="/signup" style={styles.link}>
            Sign up
          </Link>
        </div>
      </form>
    </div>
  );
}

// Basic inline styles to get you started (swap with Tailwind or CSS Modules later)
const styles = {
  container: {
    display: "flex",
    height: "100vh",
    justifyContent: "center",
    alignItems: "center",
    backgroundColor: "#f4f4f5",
  },
  form: {
    display: "flex",
    flexDirection: "column" as const,
    width: "100%",
    maxWidth: "400px",
    padding: "2rem",
    backgroundColor: "white",
    borderRadius: "8px",
    boxShadow: "0 4px 6px -1px rgb(0 0 0 / 0.1)",
  },
  title: {
    marginTop: 0,
    marginBottom: "1.5rem",
    textAlign: "center" as const,
    color: "#18181b",
  },
  error: {
    padding: "0.75rem",
    marginBottom: "1rem",
    backgroundColor: "#fee2e2",
    color: "#991b1b",
    borderRadius: "4px",
    fontSize: "0.875rem",
  },
  inputGroup: { marginBottom: "1rem" },
  label: {
    display: "block",
    marginBottom: "0.5rem",
    fontSize: "0.875rem",
    fontWeight: 500,
    color: "#3f3f46",
  },
  input: {
    width: "100%",
    padding: "0.5rem",
    fontSize: "1rem",
    border: "1px solid #d4d4d8",
    borderRadius: "4px",
    boxSizing: "border-box" as const,
  },
  button: {
    marginTop: "1rem",
    padding: "0.75rem",
    fontSize: "1rem",
    fontWeight: 600,
    color: "white",
    backgroundColor: "#2563eb",
    border: "none",
    borderRadius: "4px",
    cursor: "pointer",
  },
  footer: {
    marginTop: "1.5rem",
    textAlign: "center" as const,
    fontSize: "0.875rem",
    color: "#71717a",
  },
  link: { color: "#2563eb", textDecoration: "none", fontWeight: 500 },
};
