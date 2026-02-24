import { EditorView } from "@codemirror/view";
import CodeMirror from "@uiw/react-codemirror";
import { useMemo, useRef } from "react";
import { cn } from "@/lib/utils";
import {
	handlebarsAutocomplete,
	handlebarsHighlight,
	handlebarsTheme,
} from "./handlebarsExtension";

interface Props {
	label: string;
	defaultValue: string;
	loadId: number;
	placeholder?: string;
	namespaces: string[];
	namespaceFields: Record<string, string[]>;
	onChange: (value: string) => void;
	optional?: boolean;
}

export default function FieldEditor({
	label,
	defaultValue,
	loadId,
	placeholder,
	namespaces,
	namespaceFields,
	onChange,
	optional = false,
}: Props) {
	const initialValueRef = useRef(defaultValue);

	const extensions = useMemo(
		() => [
			handlebarsHighlight,
			handlebarsAutocomplete(namespaces, namespaceFields),
			handlebarsTheme,
			EditorView.lineWrapping,
		],
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[namespaces, namespaceFields],
	);

	return (
		<div className="space-y-1">
			<p
				className={cn(
					"text-xs font-medium",
					optional && "text-muted-foreground",
				)}
			>
				{label}
				{optional && (
					<span className="ml-1 text-xs text-muted-foreground">(optional)</span>
				)}
			</p>
			<div className="rounded-md border bg-background focus-within:ring-1 focus-within:ring-ring">
				<CodeMirror
					key={loadId}
					value={initialValueRef.current}
					placeholder={placeholder}
					extensions={extensions}
					onChange={onChange}
					basicSetup={{
						lineNumbers: false,
						foldGutter: false,
						highlightActiveLine: false,
						highlightSelectionMatches: false,
						autocompletion: false,
						searchKeymap: false,
					}}
				/>
			</div>
		</div>
	);
}
