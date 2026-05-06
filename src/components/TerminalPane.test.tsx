import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createTerminalSession, markRendererDegraded } from "../domain/terminal";
import { notifyTerminalPaneRendererDegraded, notifyTerminalPaneResize, TerminalPane } from "./TerminalPane";

describe("TerminalPane", () => {
  it("shows WebGL renderer health for preferred terminal sessions", () => {
    const session = createTerminalSession({
      id: "ses-webgl",
      title: "1. shell",
      cwd: "/repo",
      branch: "main",
      seedLines: ["$ echo hello"],
    });

    render(<TerminalPane session={session} />);

    expect(screen.getByLabelText("Terminal pane 1. shell")).toBeInTheDocument();
    expect(screen.getByText("WebGL")).toBeInTheDocument();
    expect(screen.getByText("$ echo hello")).toBeInTheDocument();
  });

  it("keeps terminal text visible when the renderer is degraded", () => {
    const session = markRendererDegraded(
      createTerminalSession({
        id: "ses-fallback",
        title: "2. fallback",
        cwd: "/repo",
        branch: "main",
        seedLines: ["raw PTY bytes remain visible"],
      }),
      "WebGL context unavailable",
    );

    render(<TerminalPane session={session} />);

    expect(screen.getByText("Fallback")).toBeInTheDocument();
    expect(screen.getByText("WebGL context unavailable")).toBeInTheDocument();
    expect(screen.getByText("raw PTY bytes remain visible")).toBeInTheDocument();
  });

  it("calls the run handler from the pane action", async () => {
    const onRun = vi.fn();
    const session = createTerminalSession({
      id: "ses-run",
      title: "3. runner",
      cwd: "/repo",
      branch: "main",
      seedLines: [],
    });

    render(<TerminalPane session={session} onRun={onRun} />);
    fireEvent.click(screen.getByRole("button", { name: "Run 3. runner" }));

    expect(onRun).toHaveBeenCalledWith(session);
  });

  it("calls the close handler from the pane action", async () => {
    const onClose = vi.fn();
    const session = createTerminalSession({
      id: "ses-close",
      title: "4. closer",
      cwd: "/repo",
      branch: "main",
      seedLines: [],
    });

    render(<TerminalPane session={session} onClose={onClose} />);
    fireEvent.click(screen.getByRole("button", { name: "Close 4. closer" }));

    expect(onClose).toHaveBeenCalledWith(session);
  });

  it("reports fitted xterm dimensions to the resize handler", () => {
    const onResize = vi.fn();
    const session = createTerminalSession({
      id: "ses-resize",
      ptyId: "pty-resize",
      title: "4. resizable",
      cwd: "/repo",
      branch: "main",
      seedLines: [],
    });

    notifyTerminalPaneResize(
      session,
      {
        proposeDimensions: () => ({ cols: 132, rows: 40 }),
      },
      onResize,
    );

    expect(onResize).toHaveBeenCalledWith(session, 132, 40);
  });

  it("reports renderer degradation to the pane owner", () => {
    const onRendererDegraded = vi.fn();
    const session = createTerminalSession({
      id: "ses-renderer-health",
      title: "5. renderer",
      cwd: "/repo",
      branch: "main",
      seedLines: [],
    });

    notifyTerminalPaneRendererDegraded(session, "WebGL context lost", onRendererDegraded);

    expect(onRendererDegraded).toHaveBeenCalledWith(session, "WebGL context lost");
  });

  it("opens safe terminal links and surfaces blocked schemes", () => {
    const onOpenLink = vi.fn();
    const session = createTerminalSession({
      id: "ses-link",
      title: "6. links",
      cwd: "/repo",
      branch: "main",
      seedLines: ["server http://localhost:3000/docs", "ignored javascript:alert"],
    });

    render(<TerminalPane session={session} onOpenLink={onOpenLink} />);

    fireEvent.click(screen.getByRole("button", { name: "Open terminal link http://localhost:3000/docs" }));

    expect(onOpenLink).toHaveBeenCalledWith("http://localhost:3000/docs");
    expect(screen.getByText("Blocked terminal link javascript:alert · scheme not allowed")).toBeInTheDocument();
  });
});
