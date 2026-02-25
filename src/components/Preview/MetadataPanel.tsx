import type { PreviewRenderedEmail } from "@/lib/ipc";

interface Props {
	rendered: PreviewRenderedEmail | null;
}

function Row({ label, value }: { label: string; value: string }) {
	return (
		<div className="flex gap-2 text-xs">
			<span className="w-16 shrink-0 text-right font-medium text-muted-foreground">
				{label}
			</span>
			<span className="min-w-0 truncate">{value}</span>
		</div>
	);
}

export default function MetadataPanel({ rendered }: Props) {
	if (!rendered) return null;

	return (
		<div className="space-y-0.5 border-b bg-muted/30 px-3 py-2">
			<Row label="To" value={rendered.to} />
			{rendered.cc && <Row label="CC" value={rendered.cc} />}
			{rendered.bcc && <Row label="BCC" value={rendered.bcc} />}
			<Row label="Subject" value={rendered.subject} />
			{rendered.attachments.length > 0 && (
				<Row label="Attach" value={rendered.attachments.join(", ")} />
			)}
		</div>
	);
}
