import { EditorView } from "@codemirror/view";
import CodeMirror from "@uiw/react-codemirror";
import { useMemo, useRef } from "react";
import {
	handlebarsAutocomplete,
	handlebarsHighlight,
	handlebarsTheme,
} from "./handlebarsExtension";

interface Props {
	defaultValue: string;
	loadId: number;
	namespaces: string[];
	namespaceFields: Record<string, string[]>;
	onChange: (value: string) => void;
}

export default function BodyEditor({
	defaultValue,
	loadId,
	namespaces,
	namespaceFields,
	onChange,
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
			<p className="text-xs font-medium">Body</p>
			<div className="min-h-52 rounded-md border bg-background focus-within:ring-1 focus-within:ring-ring">
				<CodeMirror
					key={loadId}
					value={initialValueRef.current}
					height="208px"
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
