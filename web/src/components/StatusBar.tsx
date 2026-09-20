import type { ExecutionState } from "../types/execution";

interface StatusBarProps {
  status: ExecutionState;
}

export default function StatusBar({ status }: StatusBarProps) {
  const colors: Record<ExecutionState, string> = {
    IDLE: "#333",
    RUNNING: "#0066cc",
    COMPLETED: "#006600",
    FAILED: "#cc0000",
    CANCELLED: "#cc6600",
  };

  return <strong style={{ color: colors[status] }}>Status: {status}</strong>;
}
