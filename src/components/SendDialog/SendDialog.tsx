import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
	cancelSend,
	type PreviewEntryStatus,
	type SendBatchReport,
	type SendProgressEvent,
	type SourceFileSpec,
	sendBatch,
	type TemplateFields,
} from "@/lib/ipc";

interface Props {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	templatePath: string;
	templateFields: TemplateFields;
	sourceFileSpecs: SourceFileSpec[];
	profileName: string;
	entryCount: number;
	validationEntries: PreviewEntryStatus[];
}

type View =
	| { kind: "confirm" }
	| {
			kind: "sending";
			completed: number;
			total: number;
			currentRecipient: string;
	  }
	| { kind: "report"; report: SendBatchReport };

export default function SendDialog({
	open,
	onOpenChange,
	templatePath,
	templateFields,
	sourceFileSpecs,
	profileName,
	entryCount,
	validationEntries,
}: Props) {
	const [view, setView] = useState<View>({ kind: "confirm" });

	// Reset to confirm view when dialog opens.
	const prevOpen = useRef(open);
	useEffect(() => {
		if (open && !prevOpen.current) {
			setView({ kind: "confirm" });
		}
		prevOpen.current = open;
	}, [open]);

	const warningCount = validationEntries.filter((e) => !e.is_valid).length;

	// Listen for progress events during sending.
	useEffect(() => {
		if (view.kind !== "sending") return;
		let cancelled = false;
		const promise = listen<SendProgressEvent>("send-progress", (event) => {
			if (cancelled) return;
			setView({
				kind: "sending",
				completed: event.payload.completed,
				total: event.payload.total,
				currentRecipient: event.payload.recipient,
			});
		});
		return () => {
			cancelled = true;
			promise.then((unlisten) => unlisten());
		};
	}, [view.kind]);

	const handleSend = useCallback(
		async (retryIndices?: number[]) => {
			const total = retryIndices ? retryIndices.length : entryCount;
			setView({ kind: "sending", completed: 0, total, currentRecipient: "" });
			try {
				const report = await sendBatch(
					templatePath,
					templateFields,
					sourceFileSpecs,
					profileName,
					retryIndices ?? null,
				);
				setView({ kind: "report", report });
			} catch (err) {
				setView({
					kind: "report",
					report: {
						total,
						success_count: 0,
						failure_count: total,
						results: [
							{
								entry_index: 0,
								recipient: "",
								success: false,
								error: String(err),
							},
						],
					},
				});
			}
		},
		[templatePath, templateFields, sourceFileSpecs, profileName, entryCount],
	);

	const handleCancel = async () => {
		try {
			await cancelSend();
		} catch {
			// Best-effort cancellation
		}
	};

	const handleRetry = () => {
		if (view.kind !== "report") return;
		const failedIndices = view.report.results
			.filter((r) => !r.success)
			.map((r) => r.entry_index);
		handleSend(failedIndices);
	};

	// Prevent closing during send.
	const handleOpenChange = (next: boolean) => {
		if (!next && view.kind === "sending") return;
		onOpenChange(next);
	};

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogContent className="max-w-md">
				{view.kind === "confirm" && (
					<ConfirmView
						entryCount={entryCount}
						profileName={profileName}
						warningCount={warningCount}
						onSend={() => handleSend()}
						onCancel={() => onOpenChange(false)}
					/>
				)}
				{view.kind === "sending" && (
					<SendingView
						completed={view.completed}
						total={view.total}
						currentRecipient={view.currentRecipient}
						onCancel={handleCancel}
					/>
				)}
				{view.kind === "report" && (
					<ReportView
						report={view.report}
						onRetry={handleRetry}
						onClose={() => onOpenChange(false)}
					/>
				)}
			</DialogContent>
		</Dialog>
	);
}

// ── Confirm View ─────────────────────────────────────────────────────────────

function ConfirmView({
	entryCount,
	profileName,
	warningCount,
	onSend,
	onCancel,
}: {
	entryCount: number;
	profileName: string;
	warningCount: number;
	onSend: () => void;
	onCancel: () => void;
}) {
	return (
		<>
			<DialogHeader>
				<DialogTitle>Send Emails</DialogTitle>
			</DialogHeader>
			<div className="space-y-3 py-2">
				<p className="text-sm">
					Send <strong>{entryCount}</strong> email
					{entryCount !== 1 ? "s" : ""} using profile{" "}
					<strong>{profileName}</strong>?
				</p>
				{warningCount > 0 && (
					<div className="rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-800 dark:border-amber-700 dark:bg-amber-950 dark:text-amber-200">
						{warningCount} entry{warningCount !== 1 ? "ies have" : "y has"}{" "}
						validation warnings. Emails will still be sent for these entries,
						but they may contain errors.
					</div>
				)}
			</div>
			<DialogFooter>
				<Button variant="outline" size="sm" onClick={onCancel}>
					Cancel
				</Button>
				<Button size="sm" onClick={onSend}>
					Send {entryCount} email{entryCount !== 1 ? "s" : ""}
				</Button>
			</DialogFooter>
		</>
	);
}

// ── Sending View ─────────────────────────────────────────────────────────────

function SendingView({
	completed,
	total,
	currentRecipient,
	onCancel,
}: {
	completed: number;
	total: number;
	currentRecipient: string;
	onCancel: () => void;
}) {
	const pct = total > 0 ? Math.round((completed / total) * 100) : 0;
	return (
		<>
			<DialogHeader>
				<DialogTitle>Sending...</DialogTitle>
			</DialogHeader>
			<div className="space-y-3 py-2">
				<Progress value={pct} />
				<p className="text-center text-sm text-muted-foreground">
					{completed} of {total} sent
				</p>
				{currentRecipient && (
					<p className="truncate text-center text-xs text-muted-foreground">
						{currentRecipient}
					</p>
				)}
			</div>
			<DialogFooter>
				<Button variant="outline" size="sm" onClick={onCancel}>
					Cancel
				</Button>
			</DialogFooter>
		</>
	);
}

// ── Report View ──────────────────────────────────────────────────────────────

function ReportView({
	report,
	onRetry,
	onClose,
}: {
	report: SendBatchReport;
	onRetry: () => void;
	onClose: () => void;
}) {
	const failures = report.results.filter((r) => !r.success);

	return (
		<>
			<DialogHeader>
				<DialogTitle>Send Complete</DialogTitle>
			</DialogHeader>
			<div className="space-y-3 py-2">
				{report.success_count > 0 && (
					<p className="text-sm text-green-700 dark:text-green-400">
						{report.success_count} email
						{report.success_count !== 1 ? "s" : ""} sent successfully.
					</p>
				)}
				{report.failure_count > 0 && (
					<p className="text-sm text-destructive">
						{report.failure_count} email
						{report.failure_count !== 1 ? "s" : ""} failed.
					</p>
				)}
				{failures.length > 0 && (
					<ScrollArea className="max-h-48 rounded-md border">
						<div className="p-2 text-xs">
							{failures.map((f) => (
								<div
									key={f.entry_index}
									className="border-b py-1.5 last:border-b-0"
								>
									<span className="font-medium">#{f.entry_index + 1}</span>
									{f.recipient && (
										<span className="ml-1 text-muted-foreground">
											{f.recipient}
										</span>
									)}
									<p className="mt-0.5 text-destructive">{f.error}</p>
								</div>
							))}
						</div>
					</ScrollArea>
				)}
			</div>
			<DialogFooter>
				{failures.length > 0 && (
					<Button variant="outline" size="sm" onClick={onRetry}>
						Retry Failed ({failures.length})
					</Button>
				)}
				<Button size="sm" onClick={onClose}>
					Close
				</Button>
			</DialogFooter>
		</>
	);
}
