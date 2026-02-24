import {
	autocompletion,
	type Completion,
	type CompletionContext,
} from "@codemirror/autocomplete";
import {
	Decoration,
	type DecorationSet,
	EditorView,
	MatchDecorator,
	ViewPlugin,
	type ViewUpdate,
} from "@codemirror/view";

// ── Syntax highlighting ───────────────────────────────────────────────────────

/** Matches complete {{...}} expressions including block helpers. */
const HB_REGEX = /\{\{[^}]*\}\}/g;

const hbMark = Decoration.mark({ class: "cm-hb-expression" });

const hbDecorator = new MatchDecorator({
	regexp: HB_REGEX,
	decoration: () => hbMark,
});

export const handlebarsHighlight = ViewPlugin.fromClass(
	class {
		decorations: DecorationSet;
		constructor(view: EditorView) {
			this.decorations = hbDecorator.createDeco(view);
		}
		update(update: ViewUpdate) {
			this.decorations = hbDecorator.updateDeco(update, this.decorations);
		}
	},
	{ decorations: (v) => v.decorations },
);

// ── Autocomplete ──────────────────────────────────────────────────────────────

function makeHandlebarsCompletions(
	namespaces: string[],
	namespaceFields: Record<string, string[]>,
) {
	return (ctx: CompletionContext) => {
		// Match from {{ to current cursor position (no closing }})
		const before = ctx.matchBefore(/\{\{[^}]*/);
		if (!before) return null;

		const inside = before.text.slice(2); // strip leading {{
		const dotIdx = inside.indexOf(".");

		// Explicit apply: insert label and position cursor after it.
		const applyCompletion =
			(label: string) =>
			(view: EditorView, _completion: Completion, from: number, to: number) => {
				view.dispatch({
					changes: { from, to, insert: label },
					selection: { anchor: from + label.length },
				});
			};

		if (dotIdx === -1) {
			// No dot yet — offer namespace names.
			// Skip any leading whitespace after {{ so `{{ cla` works too.
			const identStart = inside.search(/\S/);
			const wordFrom =
				identStart === -1
					? before.from + 2 + inside.length // cursor right after {{
					: before.from + 2 + identStart; // start of non-space text
			const options: Completion[] = namespaces.map((ns) => ({
				label: ns,
				type: "variable",
				detail: "namespace",
				apply: applyCompletion(ns),
			}));
			return {
				from: wordFrom,
				options,
				validFor: /^[a-zA-Z0-9_]*$/,
			};
		}

		// After a dot — offer field names for the resolved namespace
		const ns = inside.slice(0, dotIdx).trim();
		const fields = namespaceFields[ns] ?? [];
		const options: Completion[] = fields.map((f) => ({
			label: f,
			type: "property",
			detail: `${ns} field`,
			apply: applyCompletion(f),
		}));
		return {
			from: before.from + 2 + dotIdx + 1,
			options,
			validFor: /^[a-zA-Z0-9_]*$/,
		};
	};
}

export function handlebarsAutocomplete(
	namespaces: string[],
	namespaceFields: Record<string, string[]>,
) {
	return autocompletion({
		override: [makeHandlebarsCompletions(namespaces, namespaceFields)],
		activateOnTyping: true,
	});
}

// ── Theme ─────────────────────────────────────────────────────────────────────

export const handlebarsTheme = EditorView.theme({
	"&": {
		fontSize: "0.875rem",
		fontFamily: "inherit",
		backgroundColor: "transparent",
	},
	".cm-content": {
		padding: "6px 8px",
	},
	"&.cm-focused": {
		outline: "none",
	},
	".cm-line": {
		lineHeight: "1.5",
	},
});
