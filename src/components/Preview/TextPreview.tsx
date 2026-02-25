interface Props {
	text: string;
}

export default function TextPreview({ text }: Props) {
	return (
		<pre className="whitespace-pre-wrap p-3 font-mono text-xs leading-relaxed">
			{text}
		</pre>
	);
}
