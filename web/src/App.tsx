import { useState, useRef } from "react";
import {
  createExecution,
  cancelExecution,
  connectExecutionStream,
} from "./api/execution";
import { type ExecutionState, EventType } from "./types/execution";

import Editor from "./components/Editor";
import OutputPanel from "./components/OutputPanel";
import RunButton from "./components/RunButton";
import StatusBar from "./components/StatusBar";

export default function App() {
  const [code, setCode] = useState(
    'import time\nprint("Hello from Monaco!", flush=True)\ntime.sleep(1)\nprint("Done!")',
  );
  const [language, setLanguage] = useState("python");
  const [output, setOutput] = useState("");
  const [status, setStatus] = useState<ExecutionState>("IDLE");

  const activeJobId = useRef<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const handleRun = async () => {
    setOutput("");
    setStatus("RUNNING");

    try {
      const jobId = await createExecution(code, language);
      activeJobId.current = jobId;

      wsRef.current = connectExecutionStream(
        jobId,
        (msg) => {
          if (
            (msg.type === EventType.STDOUT || msg.type === EventType.STDERR) &&
            msg.data
          ) {
            const data = msg.data;
            setOutput((prev) => prev + atob(data));
          } else if (msg.type === EventType.FINISHED) {
            setStatus("COMPLETED");
          }
        },
        () =>
          setStatus((current) =>
            current === "RUNNING" ? "COMPLETED" : current,
          ),
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
      if (wsRef.current) wsRef.current.close();
    } catch (err) {
      console.error("Failed to cancel:", err);
    }
  };

  return (
    <div
      style={{
        padding: "2rem",
        maxWidth: "1200px",
        margin: "0 auto",
        fontFamily: "sans-serif",
      }}
    >
      <h2 style={{ marginBottom: "1rem" }}>Polyglot Runtime</h2>

      <div
        style={{
          display: "flex",
          gap: "1rem",
          height: "500px",
          marginBottom: "1rem",
        }}
      >
        <Editor
          code={code}
          language={language}
          disabled={status === "RUNNING"}
          onChange={(val) => setCode(val || "")}
          onLanguageChange={setLanguage}
        />
        <OutputPanel output={output} />
      </div>

      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <RunButton status={status} onRun={handleRun} onCancel={handleCancel} />
        <StatusBar status={status} />
      </div>
    </div>
  );
}
