import { SettingsIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { CsvPreviewResult, SourceSlot, TemplateInfo } from "@/lib/ipc";
import SourceConfigDialog from "./SourceConfigDialog";
import SourceSlotRow from "./SourceSlotRow";

export interface SourceState {
	path: string;
	csvPreview: CsvPreviewResult | null;
	separatorOverride: string | null;
	encodingOverride: string | null;
	error: string | null;
	formFields: string[] | null;
	formValues: Record<string, string> | null;
}

interface Props {
	templateInfo: TemplateInfo | null;
	sourcesState: Record<string, SourceState>;
	sourceConfigOpen: boolean;
	onSourceConfigOpenChange: (open: boolean) => void;
	onSourcesChange: (sources: SourceSlot[]) => void;
	onFileSelect: (namespace: string, path: string) => void;
	onSeparatorChange: (namespace: string, sep: string) => void;
	onEncodingChange: (namespace: string, enc: string) => void;
	onFormValueChange: (namespace: string, field: string, value: string) => void;
}

export default function DataPanel({
	templateInfo,
	sourcesState,
	sourceConfigOpen,
	onSourceConfigOpenChange,
	onSourcesChange,
	onFileSelect,
	onSeparatorChange,
	onEncodingChange,
	onFormValueChange,
}: Props) {
	if (!templateInfo) {
		return (
			<div className="flex h-full items-center justify-center p-4 text-center">
				<p className="text-sm text-muted-foreground">
					Open a template to see data sources
				</p>
			</div>
		);
	}

	return (
		<>
			<ScrollArea className="h-full">
				<div className="py-2">
					<div className="flex items-center justify-between px-3 pb-1">
						<p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
							Data Sources
						</p>
						<Button
							size="sm"
							variant="ghost"
							className="h-6 w-6 p-0"
							onClick={() => onSourceConfigOpenChange(true)}
						>
							<SettingsIcon className="h-3.5 w-3.5" />
						</Button>
					</div>
					{templateInfo.sources.map((slot, i) => (
						<div key={slot.namespace}>
							{i > 0 && <Separator className="my-1" />}
							<SourceSlotRow
								slot={slot}
								state={sourcesState[slot.namespace]}
								onFileSelect={onFileSelect}
								onSeparatorChange={onSeparatorChange}
								onEncodingChange={onEncodingChange}
								onFormValueChange={onFormValueChange}
							/>
						</div>
					))}
				</div>
			</ScrollArea>
			<SourceConfigDialog
				open={sourceConfigOpen}
				onOpenChange={onSourceConfigOpenChange}
				sources={templateInfo.sources}
				onSave={onSourcesChange}
			/>
		</>
	);
}
