import type { FitAddon as FitAddonInstance } from "@xterm/addon-fit";
import type { Terminal as XtermTerminalInstance } from "@xterm/xterm";
import { ExternalLink, Play, ShieldAlert, SplitSquareHorizontal, XCircle } from "lucide-react";
import { useEffect, useRef } from "react";
import { HC_XTERM_THEME } from "../design/haneulchiDesignTokens";
import { getRendererHealthLabel, type TerminalSession } from "../domain/terminal";
import { extractTerminalLinks } from "../domain/terminalLinks";
import "@xterm/xterm/css/xterm.css";

interface TerminalPaneProps {
  session: TerminalSession;
  highlighted?: boolean;
  onRun?: (session: TerminalSession) => void;
  onInput?: (session: TerminalSession, input: string) => void;
  onResize?: (session: TerminalSession, cols: number, rows: number) => void;
  onClose?: (session: TerminalSession) => void;
  onSplit?: (session: TerminalSession) => void;
  onRendererDegraded?: (session: TerminalSession, reason: string) => void;
  onOpenLink?: (url: string) => void;
}

interface TerminalFitLike {
  proposeDimensions?: () => { cols: number; rows: number } | undefined;
}

export function notifyTerminalPaneResize(
  session: TerminalSession,
  fitAddon: TerminalFitLike | undefined,
  onResize?: (session: TerminalSession, cols: number, rows: number) => void,
) {
  const dimensions = fitAddon?.proposeDimensions?.();
  if (!dimensions) return;
  onResize?.(session, dimensions.cols, dimensions.rows);
}

export function notifyTerminalPaneRendererDegraded(
  session: TerminalSession,
  reason: string,
  onRendererDegraded?: (session: TerminalSession, reason: string) => void,
) {
  onRendererDegraded?.(session, reason);
}

export function TerminalPane({
  session,
  highlighted = false,
  onRun,
  onInput,
  onResize,
  onClose,
  onSplit,
  onRendererDegraded,
  onOpenLink,
}: TerminalPaneProps) {
  const terminalRootRef = useRef<HTMLDivElement | null>(null);
  const canMountXterm =
    import.meta.env.MODE !== "test" &&
    session.renderer.kind === "webgl" &&
    typeof window !== "undefined" &&
    typeof ResizeObserver !== "undefined";
  const rendererLabel = getRendererHealthLabel(session.renderer);
  const terminalLinks = extractTerminalLinks(session.lines);

  useEffect(() => {
    if (!canMountXterm || !terminalRootRef.current) return;

    let cancelled = false;
    let terminal: XtermTerminalInstance | undefined;
    let fitAddon: FitAddonInstance | undefined;
    let resizeObserver: ResizeObserver | undefined;
    const terminalRoot = terminalRootRef.current;
    const handleWebglContextLost = (event: Event) => {
      event.preventDefault();
      notifyTerminalPaneRendererDegraded(session, "WebGL context lost", onRendererDegraded);
    };

    terminalRoot.addEventListener("webglcontextlost", handleWebglContextLost, true);

    void Promise.all([import("@xterm/xterm"), import("@xterm/addon-fit"), import("@xterm/addon-webgl")]).then(
      ([{ Terminal: XtermTerminal }, { FitAddon }, { WebglAddon }]) => {
        if (cancelled || !terminalRootRef.current) return;

        terminal = new XtermTerminal({
          allowProposedApi: false,
          convertEol: true,
          cursorBlink: true,
          cursorStyle: "bar",
          disableStdin: false,
          fontFamily: '"JetBrains Mono", "SF Mono", ui-monospace, Menlo, Monaco, Consolas, monospace',
          fontSize: 13,
          lineHeight: 1.46,
          scrollback: 10000,
          theme: HC_XTERM_THEME,
        });
        fitAddon = new FitAddon();
        terminal.loadAddon(fitAddon);

        try {
          terminal.loadAddon(new WebglAddon());
        } catch (error) {
          // xterm still renders with its default renderer when WebGL is unavailable.
          const detail = error instanceof Error && error.message ? error.message : "using canvas fallback";
          notifyTerminalPaneRendererDegraded(session, `WebGL renderer unavailable · ${detail}`, onRendererDegraded);
        }

        terminal.open(terminalRootRef.current);
        session.lines.forEach((line) => terminal?.writeln(line));
        terminal.onData((data) => onInput?.(session, data));

        resizeObserver = new ResizeObserver(() => {
          try {
            fitAddon?.fit();
            notifyTerminalPaneResize(session, fitAddon, onResize);
          } catch {
            // The pane can be temporarily hidden during layout changes.
          }
        });
        resizeObserver.observe(terminalRootRef.current);
      },
    ).catch((error) => {
      if (cancelled) return;
      const detail = error instanceof Error && error.message ? error.message : "xterm renderer modules unavailable";
      notifyTerminalPaneRendererDegraded(session, `renderer module load failed · ${detail}`, onRendererDegraded);
    },
    );

    return () => {
      cancelled = true;
      terminalRoot.removeEventListener("webglcontextlost", handleWebglContextLost, true);
      resizeObserver?.disconnect();
      terminal?.dispose();
    };
  }, [canMountXterm, onInput, onRendererDegraded, onResize, session, session.id, session.lines]);

  return (
    <article
      className={`hc-terminal-pane ${highlighted ? "is-highlighted" : ""}`}
      aria-current={highlighted ? "true" : undefined}
      aria-label={`Terminal pane ${session.title}`}
    >
      <header>
        <span className={`hc-status-dot ${session.status}`} />
        <strong>{session.title}</strong>
        <span className={`hc-renderer-badge ${session.renderer.degraded ? "degraded" : ""}`}>{rendererLabel}</span>
        <div>
          <button type="button" aria-label={`Split ${session.title}`} onClick={() => onSplit?.(session)}>
            <SplitSquareHorizontal size={14} />
          </button>
          <button type="button" aria-label={`Run ${session.title}`} onClick={() => onRun?.(session)}>
            <Play size={14} />
          </button>
          <button type="button" aria-label={`Close ${session.title}`} onClick={() => onClose?.(session)}>
            <XCircle size={14} />
          </button>
        </div>
      </header>
      <div className="hc-xterm-host" ref={terminalRootRef} aria-hidden={!canMountXterm} />
      <pre className={canMountXterm ? "hc-terminal-preview sr-only" : "hc-terminal-preview"}>
        {session.renderer.reason ? <span className="hc-renderer-reason">{session.renderer.reason}</span> : null}
        {session.lines.map((line) => (
          <span key={line}>{line}</span>
        ))}
      </pre>
      {terminalLinks.length > 0 ? (
        <div className="hc-terminal-links" aria-label={`Terminal links ${session.title}`}>
          {terminalLinks.map((link) =>
            link.status === "safe" ? (
              <button type="button" key={link.url} aria-label={`Open terminal link ${link.url}`} onClick={() => onOpenLink?.(link.url)}>
                <ExternalLink size={13} />
                <span>{link.url}</span>
              </button>
            ) : (
              <span className="hc-terminal-link-blocked" key={link.url}>
                <ShieldAlert size={13} />
                Blocked terminal link {link.url} · {link.reason}
              </span>
            ),
          )}
        </div>
      ) : null}
    </article>
  );
}
