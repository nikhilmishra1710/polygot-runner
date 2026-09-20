import type { StreamMessage } from "../types/execution";

const API_HTTP_BASE = "http://localhost:8080/v1";
const API_WS_BASE = "ws://localhost:8080/v1";

export async function createExecution(
  code: string,
  language: string = "python",
): Promise<string> {
  const response = await fetch(`${API_HTTP_BASE}/executions`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      language,
      files: [{ path: "main.py", contents: btoa(code) }],
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to create execution: ${response.statusText}`);
  }

  const data = await response.json();
  return data.id;
}

export async function cancelExecution(jobId: string): Promise<void> {
  const response = await fetch(`${API_HTTP_BASE}/executions/${jobId}/cancel`, {
    method: "POST",
  });

  if (!response.ok && response.status !== 204) {
    throw new Error("Failed to cancel execution");
  }
}

export function connectExecutionStream(
  jobId: string,
  onMessage: (msg: StreamMessage) => void,
  onClose: () => void,
  onError: (err: Event) => void,
): WebSocket {
  const ws = new WebSocket(`${API_WS_BASE}/executions/${jobId}/stream`);

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data) as StreamMessage;
      onMessage(msg);
    } catch (err) {
      console.error("Failed to parse WS message", err);
    }
  };

  ws.onerror = onError;
  ws.onclose = onClose;

  return ws;
}
