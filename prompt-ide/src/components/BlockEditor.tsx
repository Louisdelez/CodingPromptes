import { useEffect, useRef } from 'react';
import { EditorView, keymap, lineNumbers, placeholder as cmPlaceholder } from '@codemirror/view';
import { EditorState } from '@codemirror/state';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { oneDark } from '@codemirror/theme-one-dark';
import { promptHighlighting, promptAutoComplete } from '../lib/codemirror-prompt';
import { useTheme } from '../lib/theme';

const lightTheme = EditorView.theme({
  '&': { backgroundColor: 'transparent' },
  '.cm-content': { minHeight: '60px', color: '#111827', caretColor: '#111827' },
  '.cm-cursor': { borderLeftColor: '#111827' },
  '.cm-scroller': { overflow: 'auto' },
  '.cm-gutters': { backgroundColor: 'transparent', color: '#9ca3af', borderRight: '1px solid #d1d5db' },
  '.cm-activeLine': { backgroundColor: 'rgba(79, 70, 229, 0.06) !important' },
  '.cm-selectionBackground': { backgroundColor: 'rgba(79, 70, 229, 0.15) !important' },
});

const darkCustomTheme = EditorView.theme({
  '&': { backgroundColor: 'transparent' },
  '.cm-content': { minHeight: '60px' },
  '.cm-scroller': { overflow: 'auto' },
});

interface BlockEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  variables?: string[];
}

export function BlockEditor({ value, onChange, placeholder, variables = [] }: BlockEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const variablesRef = useRef(variables);
  variablesRef.current = variables;
  const resolvedTheme = useTheme();

  // Recreate editor when theme changes
  useEffect(() => {
    if (!containerRef.current) return;

    // Preserve current content if editor exists
    const currentDoc = viewRef.current?.state.doc.toString() ?? value;

    // Destroy old editor
    if (viewRef.current) {
      viewRef.current.destroy();
      viewRef.current = null;
    }

    const isDark = resolvedTheme === 'dark';

    const state = EditorState.create({
      doc: currentDoc,
      extensions: [
        keymap.of([...defaultKeymap, ...historyKeymap]),
        history(),
        lineNumbers(),
        ...(isDark ? [oneDark, darkCustomTheme] : [lightTheme]),
        EditorView.lineWrapping,
        ...(placeholder ? [cmPlaceholder(placeholder)] : []),
        ...promptHighlighting,
        promptAutoComplete(() => variablesRef.current),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            onChangeRef.current(update.state.doc.toString());
          }
        }),
      ],
    });

    const view = new EditorView({
      state,
      parent: containerRef.current,
    });

    viewRef.current = view;
    return () => view.destroy();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [resolvedTheme]);

  // Sync external value changes
  useEffect(() => {
    const view = viewRef.current;
    if (view && view.state.doc.toString() !== value) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: value },
      });
    }
  }, [value]);

  return <div ref={containerRef} className="w-full" />;
}
