// App.tsx
import {
  createBrowserRouter,
  RouterProvider,
  Navigate,
} from "react-router-dom";
import { AuthProvider } from "./context/AuthContext";
import ProtectedRoute from "./routes/ProtectedRoutes";
import PublicRoute from "./routes/PublicRoutes";

import Login from "./pages/Login";
import Signup from "./pages/Signup";
import IDE from "./pages/IDE";

const router = createBrowserRouter([
  {
    // Public routes (Login & Signup)
    element: <PublicRoute />,
    children: [
      { path: "/login", element: <Login /> },
      { path: "/signup", element: <Signup /> },
    ],
  },
  {
    element: <ProtectedRoute />,
    children: [{ path: "/ide", element: <IDE /> }],
  },
  {
    path: "*",
    element: <Navigate to="/ide" replace />,
  },
]);

export default function App() {
  return (
    // The AuthProvider wraps the Router so all routes have access to the session state
    <AuthProvider>
      <RouterProvider router={router} />
    </AuthProvider>
  );
}
