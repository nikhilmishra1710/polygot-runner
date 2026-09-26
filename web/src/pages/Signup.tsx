// src/pages/Signup.tsx
import { useState } from "react";
import { Link } from "react-router-dom";
import { useAuth } from "../context/AuthContext";

export default function Signup() {
  const { signup } = useAuth();
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);
    const username = formData.get("username") as string;
    const password = formData.get("password") as string;
    const confirmPassword = formData.get("confirmPassword") as string;

    if (!username || !password) {
      setError("Username and password are required.");
      setIsSubmitting(false);
      return;
    }

    if (password !== confirmPassword) {
      setError("Passwords do not match.");
      setIsSubmitting(false);
      return;
    }

    if (password.length < 8) {
      setError("Password must be at least 8 characters long.");
      setIsSubmitting(false);
      return;
    }

    try {
      await signup(username, password);
      // The AuthProvider logs the user in, and PublicRoute redirects them to /ide
    } catch (err: any) {
      // Handle the 409 Conflict from the Go backend gracefully
      if (err.response?.status === 409) {
        setError("This username is already taken. Please choose another.");
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
        <h2 style={styles.title}>Create an Account</h2>

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
            autoComplete="new-password"
            disabled={isSubmitting}
            style={styles.input}
            required
          />
        </div>

        <div style={styles.inputGroup}>
          <label htmlFor="confirmPassword" style={styles.label}>
            Confirm Password
          </label>
          <input
            id="confirmPassword"
            name="confirmPassword"
            type="password"
            autoComplete="new-password"
            disabled={isSubmitting}
            style={styles.input}
            required
          />
        </div>

        <button type="submit" disabled={isSubmitting} style={styles.button}>
          {isSubmitting ? "Creating account..." : "Sign Up"}
        </button>

        <div style={styles.footer}>
          Already have an account?{" "}
          <Link to="/login" style={styles.link}>
            Sign in
          </Link>
        </div>
      </form>
    </div>
  );
}

// Reusing the same inline styles for consistency
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
