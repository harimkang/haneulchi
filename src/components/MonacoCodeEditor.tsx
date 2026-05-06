import Editor, { DiffEditor } from "@monaco-editor/react";

interface MonacoCodeEditorProps {
  language?: string | null;
  path: string;
  value: string;
  onChange?: (value: string) => void;
}

export function MonacoCodeEditor({ language, path, value, onChange }: MonacoCodeEditorProps) {
  const editorLanguage = language || "plaintext";

  return (
    <section className="hc-monaco-code-editor" aria-label="Monaco code editor" data-language={editorLanguage}>
      <Editor
        height="160px"
        language={editorLanguage}
        path={path}
        theme="vs-dark"
        value={value}
        onChange={(nextValue) => {
          if (typeof nextValue === "string") onChange?.(nextValue);
        }}
        options={{
          readOnly: !onChange,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          wordWrap: "on",
          fontFamily: "JetBrains Mono, SF Mono, ui-monospace, Menlo, Monaco, Consolas, monospace",
          fontSize: 12,
          lineNumbersMinChars: 3,
          renderLineHighlight: "none",
          automaticLayout: true,
        }}
        loading={<pre className="hc-monaco-loading">{value}</pre>}
      />
      {onChange ? (
        <textarea
          className="hc-monaco-editor-buffer"
          aria-label="Monaco editor buffer"
          value={value}
          onChange={(event) => onChange(event.currentTarget.value)}
          spellCheck={false}
        />
      ) : null}
      <pre className="hc-monaco-editor-fallback" aria-hidden="true">
        {value}
      </pre>
    </section>
  );
}

interface MonacoDiffEditorProps {
  path?: string | null;
  body: string;
}

export function MonacoDiffEditor({ path, body }: MonacoDiffEditorProps) {
  const language = languageFromPath(path);
  const { original, modified } = splitUnifiedDiff(body);

  return (
    <section className="hc-monaco-diff-editor" aria-label="Monaco diff editor" data-path={path ?? "workspace"} data-language={language}>
      <DiffEditor
        height="180px"
        language={language}
        original={original}
        modified={modified}
        theme="vs-dark"
        options={{
          readOnly: true,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          renderSideBySide: true,
          wordWrap: "on",
          fontFamily: "JetBrains Mono, SF Mono, ui-monospace, Menlo, Monaco, Consolas, monospace",
          fontSize: 12,
          lineNumbersMinChars: 3,
          renderLineHighlight: "none",
          automaticLayout: true,
        }}
        loading={<pre className="hc-monaco-loading">{body || "No local diff"}</pre>}
      />
      <pre className="hc-monaco-editor-fallback" aria-hidden="true">
        {body || "No local diff"}
      </pre>
    </section>
  );
}

function splitUnifiedDiff(body: string): { original: string; modified: string } {
  const original: string[] = [];
  const modified: string[] = [];

  for (const line of body.split(/\r?\n/)) {
    if (
      line.startsWith("diff --git ") ||
      line.startsWith("index ") ||
      line.startsWith("--- ") ||
      line.startsWith("+++ ") ||
      line.startsWith("@@")
    ) {
      continue;
    }

    if (line.startsWith("-")) {
      original.push(line.slice(1));
      continue;
    }

    if (line.startsWith("+")) {
      modified.push(line.slice(1));
      continue;
    }

    const content = line.startsWith(" ") ? line.slice(1) : line;
    original.push(content);
    modified.push(content);
  }

  return {
    original: original.join("\n"),
    modified: modified.join("\n"),
  };
}

function languageFromPath(path?: string | null): string {
  if (!path) return "plaintext";
  const lower = path.toLowerCase();
  if (lower.endsWith(".ts") || lower.endsWith(".tsx")) return "typescript";
  if (lower.endsWith(".js") || lower.endsWith(".jsx")) return "javascript";
  if (lower.endsWith(".md") || lower.endsWith(".markdown")) return "markdown";
  if (lower.endsWith(".json")) return "json";
  if (lower.endsWith(".css")) return "css";
  if (lower.endsWith(".html")) return "html";
  return "plaintext";
}
