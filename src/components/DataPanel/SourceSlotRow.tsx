import { open } from "@tauri-apps/plugin-dialog";
import type { SourceState } from "@/components/DataPanel/index";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import type { SourceSlot } from "@/lib/ipc";
import CsvConfigPanel from "./CsvConfigPanel";

interface Props {
	slot: SourceSlot;
	state: SourceState | undefined;
	onFileSelect: (namespace: string, path: string) => void;
	onSeparatorChange: (namespace: string, sep: string) => void;
	onEncodingChange: (namespace: string, enc: string) => void;
}

function StatusIcon({ state }: { state: SourceState | undefined }) {
	if (!state) {
		return (
			<Tooltip>
				<TooltipTrigger asChild>
					<span className="text-base leading-none text-amber-500">⚠</span>
				</TooltipTrigger>
				<TooltipContent>No file loaded</TooltipContent>
			</Tooltip>
		);
	}
	if (state.error) {
		return (
			<Tooltip>
				<TooltipTrigger asChild>
					<span className="text-base leading-none text-destructive">✕</span>
				</TooltipTrigger>
				<TooltipContent>{state.error}</TooltipContent>
			</Tooltip>
		);
	}
	return (
		<Tooltip>
			<TooltipTrigger asChild>
				<span className="text-base leading-none text-green-600">✓</span>
			</TooltipTrigger>
			<TooltipContent>Loaded</TooltipContent>
		</Tooltip>
	);
}

export default function SourceSlotRow({
	slot,
	state,
	onFileSelect,
	onSeparatorChange,
	onEncodingChange,
}: Props) {
	const handleSelect = async () => {
		const selected = await open({
			multiple: false,
			filters: [
				{
					name: "Data Files",
					extensions: ["csv", "json", "yaml", "yml", "toml"],
				},
			],
		});
		if (typeof selected === "string") {
			onFileSelect(slot.namespace, selected);
		}
	};

	const isCsv = state?.path.toLowerCase().endsWith(".csv");

	return (
		<div className="px-3 py-2">
			<div className="flex items-center gap-2">
				<StatusIcon state={state} />
				<span className="flex-1 truncate font-mono text-sm font-medium">
					{slot.namespace}
				</span>
				{slot.is_primary && (
					<Badge variant="secondary" className="text-xs">
						primary
					</Badge>
				)}
				{slot.has_join && !slot.is_primary && (
					<Badge variant="outline" className="text-xs">
						joined
					</Badge>
				)}
				{!slot.is_primary && !slot.has_join && (
					<Badge variant="outline" className="text-xs text-muted-foreground">
						global
					</Badge>
				)}
				<Button
					size="sm"
					variant="ghost"
					className="h-6 px-2 text-xs"
					onClick={handleSelect}
				>
					{state ? "Change" : "Load"}
				</Button>
			</div>

			{state && !state.error && (
				<div className="mt-0.5 pl-5 text-xs text-muted-foreground">
					{state.path.split("/").pop()}
				</div>
			)}

			{state?.error && (
				<div className="mt-0.5 pl-5 text-xs text-destructive">
					{state.error}
				</div>
			)}

			{slot.has_join && slot.join_keys.length > 0 && (
				<div className="mt-0.5 pl-5 text-xs text-muted-foreground">
					join: {slot.join_keys.join(", ")}
				</div>
			)}

			{isCsv && state?.csvPreview && (
				<div className="pl-5">
					<CsvConfigPanel
						preview={state.csvPreview}
						separatorOverride={state.separatorOverride}
						encodingOverride={state.encodingOverride}
						onSeparatorChange={(sep) => onSeparatorChange(slot.namespace, sep)}
						onEncodingChange={(enc) => onEncodingChange(slot.namespace, enc)}
					/>
				</div>
			)}
		</div>
	);
}
