import { useState, useRef } from "react";
import {
  createExecution,
  cancelExecution,
  connectExecutionStream,
} from "./api/execution";
import { type ExecutionState, EventType } from "./types/execution";

export default function App() {
  const [code, setCode] = useState(
    'import time\nprint("Hello World", flush=True)\ntime.sleep(1)\nprint("Done!")',
  );
  const [output, setOutput] = useState("");
  const [status, setStatus] = useState<ExecutionState>("IDLE");

  const activeJobId = useRef<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const handleRun = async () => {
    setOutput("");
    setStatus("RUNNING");

    try {
      const jobId = await createExecution(code, "python");
      activeJobId.current = jobId;

      wsRef.current = connectExecutionStream(
        jobId,
        (msg) => {
          if (msg.type === EventType.STDOUT || msg.type === EventType.STDERR) {
            if (msg.data) {
              setOutput((prev) => prev + atob(msg.data));
            }
          } else if (msg.type === EventType.FINISHED) {
            setStatus("COMPLETED");
          }
        },
        () => {
          setStatus((current) =>
            current === "RUNNING" ? "COMPLETED" : current,
          );
        },
        (err) => {
          console.error("WebSocket Error:", err);
          setStatus("FAILED");
        },
      );
    } catch (err) {
      console.error(err);
      setOutput(`System Error: ${err}`);
      setStatus("FAILED");
    }
  };

  const handleCancel = async () => {
    if (!activeJobId.current) return;

    try {
      await cancelExecution(activeJobId.current);
      setStatus("CANCELLED");
      if (wsRef.current) {
        wsRef.current.close();
      }
    } catch (err) {
      console.error("Failed to cancel:", err);
    }
  };

  return (
    <div
      style={{
        padding: "2rem",
        maxWidth: "900px",
        margin: "0 auto",
        fontFamily: "sans-serif",
      }}
    >
      {/* Top Bar */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          marginBottom: "0.5rem",
        }}
      >
        <strong>Polyglot Runtime</strong>
        <select disabled style={{ padding: "0.2rem" }}>
          <option>Python</option>
        </select>
      </div>

      {/* Main Workspace */}
      <div
        style={{
          display: "flex",
          gap: "1rem",
          height: "400px",
          marginBottom: "1rem",
        }}
      >
        <textarea
          value={code}
          onChange={(e) => setCode(e.target.value)}
          disabled={status === "RUNNING"}
          style={{
            flex: 1,
            fontFamily: "monospace",
            padding: "1rem",
            resize: "none",
          }}
        />

        <div
          style={{
            flex: 1,
            backgroundColor: "#1e1e1e",
            color: "#00ff00",
            padding: "1rem",
            fontFamily: "monospace",
            whiteSpace: "pre-wrap",
            overflowY: "auto",
          }}
        >
          {output || (
            <span style={{ color: "#666" }}>Output will appear here...</span>
          )}
        </div>
      </div>

      {/* Status & Controls */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <div style={{ display: "flex", gap: "1rem" }}>
          <button
            onClick={handleRun}
            disabled={status === "RUNNING"}
            style={{
              padding: "0.5rem 1.5rem",
              cursor: status === "RUNNING" ? "not-allowed" : "pointer",
            }}
          >
            Run
          </button>
          <button
            onClick={handleCancel}
            disabled={status !== "RUNNING"}
            style={{
              padding: "0.5rem 1.5rem",
              cursor: status !== "RUNNING" ? "not-allowed" : "pointer",
            }}
          >
            Cancel
          </button>
        </div>

        <strong
          style={{
            color:
              status === "RUNNING"
                ? "#0066cc"
                : status === "COMPLETED"
                  ? "#006600"
                  : status === "FAILED"
                    ? "#cc0000"
                    : status === "CANCELLED"
                      ? "#cc6600"
                      : "#333",
          }}
        >
          Status: {status}
        </strong>
      </div>
    </div>
  );
}
