// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useExecution } from "../src/hooks/useExecution";
import { EventType } from "../src/types/execution";
import { SUPPORTED_LANGUAGES } from "../src/types/language";

// Mock WebSocket Implementation
let mockWsInstance: any = null;

class MockWebSocket {
  url: string;
  onmessage: any = null;
  onclose: any = null;
  onerror: any = null;
  close = vi.fn();

  constructor(url: string) {
    this.url = url;
    mockWsInstance = this;
  }
}

describe("useExecution Hook Flow", () => {
  const pythonLang = SUPPORTED_LANGUAGES.find((l) => l.id === "python")!;

  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    vi.stubGlobal("WebSocket", MockWebSocket);
    mockWsInstance = null;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("successfully executes code and streams stdout to the state", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ id: "job-123" }),
    } as Response);

    const { result } = renderHook(() => useExecution());
    const testCode = 'print("Integration Test")';

    // 1. Run the execution
    await act(async () => {
      result.current.run(testCode, pythonLang);
    });

    // Verify API was called correctly
    expect(fetch).toHaveBeenCalledWith("http://localhost:8080/v1/executions", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        language: "python",
        files: [{ path: "main.py", contents: btoa(testCode) }],
      }),
    });

    expect(result.current.status).toBe("RUNNING");
    expect(mockWsInstance.url).toBe(
      "ws://localhost:8080/v1/executions/job-123/stream",
    );

    // 2. Stream STDOUT (Wrapped in act to prevent React warnings)
    act(() => {
      mockWsInstance.onmessage({
        data: JSON.stringify({ type: EventType.STARTED }),
      });
      mockWsInstance.onmessage({
        data: JSON.stringify({
          type: EventType.STDOUT,
          data: btoa("Integration Test\n"),
        }),
      });
    });

    expect(result.current.output).toBe("Integration Test\n");

    // 3. Complete the execution
    act(() => {
      mockWsInstance.onmessage({
        data: JSON.stringify({ type: EventType.FINISHED }),
      });
      mockWsInstance.onclose();
    });

    expect(result.current.status).toBe("COMPLETED");
  });

  it("handles cancellation", async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      json: async () => ({ id: "job-cancel-123" }),
    } as Response);

    const { result } = renderHook(() => useExecution());

    await act(async () => {
      result.current.run("while True: pass", pythonLang);
    });

    // Trigger cancel
    await act(async () => {
      await result.current.cancel();
    });

    // Verify cancellation API was called and state updated
    expect(fetch).toHaveBeenCalledWith(
      "http://localhost:8080/v1/executions/job-cancel-123/cancel",
      expect.objectContaining({ method: "POST" }),
    );
    expect(result.current.status).toBe("CANCELLED");
    expect(mockWsInstance.close).toHaveBeenCalled();
  });

  it("handles API/WebSocket failures", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ id: "job-fail-123" }),
    } as Response);

    const { result } = renderHook(() => useExecution());

    await act(async () => {
      result.current.run("print('fail')", pythonLang);
    });

    // Simulate WebSocket error
    act(() => {
      mockWsInstance.onerror(new Error("Connection refused"));
    });

    expect(result.current.status).toBe("FAILED");
  });
});
