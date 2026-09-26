// src/routes/ProtectedRoute.tsx
import { Navigate, Outlet } from "react-router-dom";
import { useAuth } from "../context/AuthContext";

export default function PublicRoute() {
  const { user } = useAuth();

  if (user) {
    return <Navigate to="/ide" replace />;
  }

  return <Outlet />;
}
