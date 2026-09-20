import type { ExecutionState } from "../types/execution";

interface RunButtonProps {
  status: ExecutionState;
  onRun: () => void;
  onCancel: () => void;
}

export default function RunButton({ status, onRun, onCancel }: RunButtonProps) {
  const isRunning = status === "RUNNING";

  return (
    <div style={{ display: "flex", gap: "1rem" }}>
      <button
        onClick={onRun}
        disabled={isRunning}
        style={{
          padding: "0.5rem 1.5rem",
          cursor: isRunning ? "not-allowed" : "pointer",
        }}
      >
        Run
      </button>
      <button
        onClick={onCancel}
        disabled={!isRunning}
        style={{
          padding: "0.5rem 1.5rem",
          cursor: !isRunning ? "not-allowed" : "pointer",
          color: "red",
        }}
      >
        Cancel
      </button>
    </div>
  );
}
