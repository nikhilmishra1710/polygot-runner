import { test, expect } from "@playwright/test";

const starter_boilerplate = "Hello from runner";

test.describe("Polyglot Runtime E2E", () => {
  test("executes python code and streams output from rust backend", async ({
    page,
  }) => {
    // Navigate to the local Vite dev server
    await page.goto("http://localhost:5173");

    // Ensure the IDE loaded
    await expect(page.getByText("Polyglot Runtime")).toBeVisible();
    await expect(page.locator(".monaco-editor")).toContainText(
      starter_boilerplate,
    );
    // 1. Interact with the real Monaco Editor reliably
    await page.locator(".monaco-editor").click();

    // Bypass OS-specific "Select All" modifiers using universal cursor highlighting
    await page.keyboard.press("End");
    await page.keyboard.press("Shift+Home");
    await page.keyboard.press("Backspace");

    // Insert the new code
    await page.keyboard.insertText(
      'import sys\nprint("Hello to stdout")\nprint("Error to stderr", file=sys.stderr)',
    );

    // FIX 2: Explicitly wait for Monaco to render the new code BEFORE clicking Run
    await expect(page.locator(".monaco-editor")).toContainText(
      "Hello to stdout",
    );

    // 2. Trigger the Execution Pipeline
    await page.getByRole("button", { name: "Run" }).click();

    // Verify UI reacts immediately
    await expect(page.getByText("Status: RUNNING")).toBeVisible();

    // 3. Wait for the Rust worker to stream the output back
    // We target the specific text inside the OutputPanel
    const outputText = page.locator("pre", { hasText: "Hello to stdout" });
    const errorText = page.locator("pre", { hasText: "Error to stderr" });

    // Increase timeout to 10s to account for potential cold-start of the Rust sandbox
    await expect(outputText).toBeVisible({ timeout: 10000 });
    await expect(errorText).toBeVisible({ timeout: 10000 });

    // 4. Verify the state machine correctly transitions to COMPLETED on WebSocket close
    await expect(page.getByText("Status: COMPLETED")).toBeVisible();
  });

  test("cancels a running execution successfully", async ({ page }) => {
    await page.goto("http://localhost:5173");
    await expect(page.locator(".monaco-editor")).toContainText(
      starter_boilerplate,
    );
    // Click the visible editor container to properly engage Monaco's focus manager
    await page.locator(".monaco-editor").click();

    await page.keyboard.press("ControlOrMeta+a");
    await page.keyboard.press("Backspace");

    // Use insertText for reliable bulk text injection
    await page.keyboard.insertText(
      "import time\nwhile True:\n  time.sleep(0.1)",
    );

    // Explicitly wait for Monaco to render the new code BEFORE clicking Run
    await expect(page.locator(".monaco-editor")).toContainText("while True:");

    await page.getByRole("button", { name: "Run" }).click();
    await expect(page.getByText("Status: RUNNING")).toBeVisible();

    // Click Cancel
    await page.getByRole("button", { name: "Cancel" }).click();

    // Verify the system safely caught the abort signal
    await expect(page.getByText("Status: CANCELLED")).toBeVisible();
  });
});
