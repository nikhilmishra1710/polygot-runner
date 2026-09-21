// src/hooks/useExecution.ts
import { useState, useRef } from "react";
import {
  createExecution,
  cancelExecution,
  connectExecutionStream,
} from "../api/execution";
import { type ExecutionState, EventType } from "../types/execution";
import { type Language } from "../types/language";

export function useExecution() {
  const [output, setOutput] = useState("");
  const [status, setStatus] = useState<ExecutionState>("IDLE");

  const activeJobId = useRef<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const run = async (code: string, language: Language) => {
    setOutput("");
    setStatus("RUNNING");

    try {
      const requestPayload = {
        language: language.id,
        files: [
          {
            path: language.fileName,
            contents: btoa(code),
          },
        ],
      };

      const jobId = await createExecution(requestPayload);
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

  const cancel = async () => {
    if (!activeJobId.current) return;
    try {
      await cancelExecution(activeJobId.current);
      setStatus("CANCELLED");
      if (wsRef.current) wsRef.current.close();
    } catch (err) {
      console.error("Failed to cancel:", err);
    }
  };

  return {
    status,
    output,
    run,
    cancel,
  };
}
