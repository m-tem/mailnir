interface Props {
	html: string | null;
}

export default function HtmlPreview({ html }: Props) {
	if (html == null) {
		return (
			<div className="flex h-full items-center justify-center p-4">
				<p className="text-xs text-muted-foreground">
					No HTML body (text-only mode)
				</p>
			</div>
		);
	}

	return (
		<iframe
			srcDoc={html}
			sandbox=""
			className="absolute inset-0 h-full w-full border-0"
			title="Email preview"
		/>
	);
}
