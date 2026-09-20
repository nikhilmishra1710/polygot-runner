// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "../src/App";
import { EventType } from "../src/types/execution";

// 1. Mock Monaco Editor for jsdom compatibility
vi.mock("@monaco-editor/react", () => {
  return {
    default: ({ value, onChange, options }: any) => (
      <textarea
        data-testid="monaco-mock"
        value={value}
        disabled={options?.readOnly}
        onChange={(e) => onChange(e.target.value)}
      />
    ),
  };
});

// 2. Mock WebSocket Implementation
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

describe("Execution Flow", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    vi.stubGlobal("WebSocket", MockWebSocket);
    mockWsInstance = null;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("executes code and streams output to the UI", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ id: "job-123" }),
    } as Response);

    render(<App />);
    const user = userEvent.setup();

    // 3. Enter Python code using the mocked editor
    const editor = screen.getByTestId("monaco-mock");
    await user.clear(editor);
    const testCode = 'print("Integration Test")';
    await user.type(editor, testCode);

    // 4. Click Run
    const runButton = screen.getByRole("button", { name: /run/i });
    await user.click(runButton);

    // 5. Verify API was called with Base64 encoded payload
    expect(fetch).toHaveBeenCalledWith("http://localhost:8080/v1/executions", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        language: "python",
        files: [{ path: "main.py", contents: btoa(testCode) }],
      }),
    });

    expect(screen.getByText("Status: RUNNING")).toBeDefined();

    // 6. Verify WebSocket connection
    await waitFor(() => {
      expect(mockWsInstance).not.toBeNull();
      expect(mockWsInstance.url).toBe(
        "ws://localhost:8080/v1/executions/job-123/stream",
      );
    });

    // 7. Simulate WebSocket streaming events
    mockWsInstance.onmessage({
      data: JSON.stringify({ type: EventType.STARTED }),
    });

    const encodedOutput = btoa("Integration Test\n");
    mockWsInstance.onmessage({
      data: JSON.stringify({ type: EventType.STDOUT, data: encodedOutput }),
    });

    mockWsInstance.onmessage({
      data: JSON.stringify({ type: EventType.FINISHED }),
    });
    mockWsInstance.onclose();

    // 8. Verify Output appears in UI
    await waitFor(() => {
      expect(screen.getByText(/Integration Test/)).toBeDefined();
    });

    expect(screen.getByText("Status: COMPLETED")).toBeDefined();
  });
});
