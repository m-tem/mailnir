import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import type { CsvPreviewResult } from "@/lib/ipc";

interface Props {
	preview: CsvPreviewResult;
	separatorOverride: string | null;
	encodingOverride: string | null;
	onSeparatorChange: (sep: string) => void;
	onEncodingChange: (enc: string) => void;
}

const SEPARATOR_OPTIONS = [
	{ value: "auto", label: "Auto" },
	{ value: ",", label: "Comma (,)" },
	{ value: ";", label: "Semicolon (;)" },
	{ value: "|", label: "Pipe (|)" },
	{ value: "\\t", label: "Tab" },
];

const ENCODING_OPTIONS = [
	{ value: "auto", label: "Auto" },
	{ value: "utf-8", label: "UTF-8" },
	{ value: "latin-1", label: "Latin-1" },
	{ value: "windows-1252", label: "Windows-1252" },
];

export default function CsvConfigPanel({
	preview,
	separatorOverride,
	encodingOverride,
	onSeparatorChange,
	onEncodingChange,
}: Props) {
	const separatorValue = separatorOverride ?? "auto";
	const encodingValue = encodingOverride ?? "auto";

	return (
		<div className="mt-2 space-y-2 rounded-md border bg-muted/30 p-2">
			<div className="flex gap-4">
				<div className="flex-1 space-y-1">
					<Label className="text-xs text-muted-foreground">Separator</Label>
					<Select value={separatorValue} onValueChange={onSeparatorChange}>
						<SelectTrigger className="h-7 text-xs">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{SEPARATOR_OPTIONS.map((opt) => (
								<SelectItem
									key={opt.value}
									value={opt.value}
									className="text-xs"
								>
									{opt.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
				<div className="flex-1 space-y-1">
					<Label className="text-xs text-muted-foreground">Encoding</Label>
					<Select value={encodingValue} onValueChange={onEncodingChange}>
						<SelectTrigger className="h-7 text-xs">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{ENCODING_OPTIONS.map((opt) => (
								<SelectItem
									key={opt.value}
									value={opt.value}
									className="text-xs"
								>
									{opt.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
			</div>

			{/* 5-row preview table */}
			{preview.headers.length > 0 && (
				<div className="overflow-x-auto rounded-sm border">
					<Table>
						<TableHeader>
							<TableRow>
								{preview.headers.map((h) => (
									<TableHead
										key={h}
										className="h-6 px-2 py-0 text-xs font-medium"
									>
										{h}
									</TableHead>
								))}
							</TableRow>
						</TableHeader>
						<TableBody>
							{preview.preview_rows.map((row, ri) => (
								<TableRow key={ri}>
									{row.map((cell, ci) => (
										<TableCell key={ci} className="px-2 py-0.5 text-xs">
											{cell}
										</TableCell>
									))}
								</TableRow>
							))}
						</TableBody>
					</Table>
				</div>
			)}

			<p className="text-xs text-muted-foreground">
				{preview.total_rows} row{preview.total_rows !== 1 ? "s" : ""} total
				{separatorOverride === null && (
					<span className="ml-1">
						· auto-detected separator:{" "}
						<code className="rounded bg-muted px-0.5">
							{preview.detected_separator === "\\t"
								? "tab"
								: preview.detected_separator}
						</code>
					</span>
				)}
			</p>
		</div>
	);
}
