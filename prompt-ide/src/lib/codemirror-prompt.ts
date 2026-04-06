import {
  ViewPlugin,
  Decoration,
  type DecorationSet,
  type EditorView,
  type ViewUpdate,
  MatchDecorator,
} from '@codemirror/view';
import { autocompletion, type CompletionContext, type CompletionResult } from '@codemirror/autocomplete';

// Decoration for {{variables}}
const variableDeco = Decoration.mark({ class: 'cm-prompt-variable' });
const xmlTagDeco = Decoration.mark({ class: 'cm-prompt-xml-tag' });
const commentDeco = Decoration.mark({ class: 'cm-prompt-comment' });
const sectionDeco = Decoration.mark({ class: 'cm-prompt-section' });

const variableMatcher = new MatchDecorator({
  regexp: /\{\{\w+\}\}/g,
  decoration: () => variableDeco,
});

const xmlMatcher = new MatchDecorator({
  regexp: /<\/?[a-zA-Z_][\w-]*(?:\s[^>]*)?>/g,
  decoration: () => xmlTagDeco,
});

const commentMatcher = new MatchDecorator({
  regexp: /\/\/.*$/gm,
  decoration: () => commentDeco,
});

const sectionMatcher = new MatchDecorator({
  regexp: /^##\s+.+$/gm,
  decoration: () => sectionDeco,
});

function createMatchPlugin(matcher: MatchDecorator) {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = matcher.createDeco(view);
      }
      update(update: ViewUpdate) {
        this.decorations = matcher.updateDeco(update, this.decorations);
      }
    },
    { decorations: (v) => v.decorations }
  );
}

export const promptHighlighting = [
  createMatchPlugin(variableMatcher),
  createMatchPlugin(xmlMatcher),
  createMatchPlugin(commentMatcher),
  createMatchPlugin(sectionMatcher),
];

// Auto-completion for variables
export function promptAutoComplete(getVariables: () => string[]) {
  return autocompletion({
    override: [
      (context: CompletionContext): CompletionResult | null => {
        const before = context.matchBefore(/\{\{?\w*/);
        if (!before) return null;

        const vars = getVariables();
        if (vars.length === 0) return null;

        return {
          from: before.from,
          options: vars.map((v) => ({
            label: `{{${v}}}`,
            type: 'variable',
            detail: 'variable',
          })),
        };
      },
    ],
  });
}
