interface Props {
	templatePath: string | null;
}

export default function TemplateEditor({ templatePath }: Props) {
	return (
		<div className="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
			<p className="text-sm font-medium text-muted-foreground">
				Template Editor
			</p>
			<p className="text-xs text-muted-foreground">Coming in Phase 7</p>
			{templatePath && (
				<code className="mt-2 max-w-full truncate rounded bg-muted px-2 py-0.5 text-xs">
					{templatePath}
				</code>
			)}
		</div>
	);
}
