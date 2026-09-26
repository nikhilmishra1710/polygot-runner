import {
  createContext,
  useContext,
  useEffect,
  useState,
  useMemo,
  type ReactNode,
} from "react";
import api from "../api/axios";

export interface User {
  id: string;
  username: string;
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  signup: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

// 2. Initialize the Context
const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // 3. Verify session on mount and listen for forced logouts
  useEffect(() => {
    const verifySession = async () => {
      try {
        const { data } = await api.get<User>("/verify");
        setUser(data);
      } catch (error) {
        setUser(null);
      } finally {
        setLoading(false);
      }
    };

    verifySession();

    // Listen to the custom event emitted by our Axios interceptor
    const handleUnauthorized = () => setUser(null);
    window.addEventListener("auth:unauthorized", handleUnauthorized);

    return () => {
      window.removeEventListener("auth:unauthorized", handleUnauthorized);
    };
  }, []);

  // 4. Define the Authentication Methods
  const login = async (username: string, password: string) => {
    await api.post("/login", { username, password });
    const { data } = await api.get<User>("/verify");
    setUser(data);
  };

  const signup = async (username: string, password: string) => {
    await api.post("/signup", { username, password });
    await login(username, password);
  };

  const logout = async () => {
    try {
      await api.post("/logout");
    } finally {
      setUser(null);
    }
  };

  const value = useMemo(
    () => ({ user, loading, login, signup, logout }),
    [user, loading],
  );

  return (
    <AuthContext.Provider value={value}>
      {!loading && children}
    </AuthContext.Provider>
  );
}

// 6. Export a custom hook for clean access in components
export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
