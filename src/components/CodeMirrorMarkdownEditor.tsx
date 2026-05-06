import { useEffect, useRef } from "react";
import { markdown } from "@codemirror/lang-markdown";
import { EditorView } from "codemirror";

interface CodeMirrorMarkdownEditorProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

export function CodeMirrorMarkdownEditor({ label, value, onChange }: CodeMirrorMarkdownEditorProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    const host = hostRef.current;
    if (!host || viewRef.current) return;

    const view = new EditorView({
      doc: value,
      parent: host,
      extensions: [
        markdown(),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            onChangeRef.current(update.state.doc.toString());
          }
        }),
      ],
    });

    viewRef.current = view;
    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, []);

  useEffect(() => {
    const view = viewRef.current;
    if (!view || view.state.doc.toString() === value) return;
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: value },
    });
  }, [value]);

  return (
    <section className="hc-codemirror-markdown-editor" aria-label="CodeMirror markdown workpad" data-language="markdown">
      <div ref={hostRef} className="hc-codemirror-host" />
      <textarea
        aria-label={label}
        className="hc-codemirror-shadow-input"
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </section>
  );
}
