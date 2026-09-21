// src/components/OutputPanel.tsx
import { type ExecutionState } from "../types/execution";

interface OutputPanelProps {
  stdout: string;
  stderr: string;
  status: ExecutionState;
}

export default function OutputPanel({
  stdout,
  stderr,
  status,
}: OutputPanelProps) {
  const hasOutput = stdout.length > 0 || stderr.length > 0;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "#1e1e1e",
        color: "#d4d4d4",
        fontFamily: "monospace",
        overflow: "auto",
        padding: "1rem",
      }}
    >
      {!hasOutput && status === "IDLE" && (
        <div style={{ color: "#808080" }}>No output yet.</div>
      )}
      {!hasOutput && status === "RUNNING" && (
        <div style={{ color: "#808080" }}>Running...</div>
      )}

      {hasOutput && (
        <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
          {stdout && (
            <div>
              <div style={{ color: "#4ec9b0", marginBottom: "0.25rem" }}>
                OUTPUT
              </div>
              <div
                style={{
                  borderBottom: "1px solid #333",
                  marginBottom: "0.5rem",
                }}
              />
              <pre style={{ margin: 0, whiteSpace: "pre-wrap" }}>{stdout}</pre>
            </div>
          )}

          {stderr && (
            <div>
              <div style={{ color: "#f48771", marginBottom: "0.25rem" }}>
                ERROR
              </div>
              <div
                style={{
                  borderBottom: "1px solid #333",
                  marginBottom: "0.5rem",
                }}
              />
              <pre
                style={{ margin: 0, whiteSpace: "pre-wrap", color: "#f48771" }}
              >
                {stderr}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
