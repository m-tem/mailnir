import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { CsvPreviewResult, TemplateInfo } from "@/lib/ipc";
import SourceSlotRow from "./SourceSlotRow";

export interface SourceState {
	path: string;
	csvPreview: CsvPreviewResult | null;
	separatorOverride: string | null;
	encodingOverride: string | null;
	error: string | null;
}

interface Props {
	templateInfo: TemplateInfo | null;
	sourcesState: Record<string, SourceState>;
	onFileSelect: (namespace: string, path: string) => void;
	onSeparatorChange: (namespace: string, sep: string) => void;
	onEncodingChange: (namespace: string, enc: string) => void;
}

export default function DataPanel({
	templateInfo,
	sourcesState,
	onFileSelect,
	onSeparatorChange,
	onEncodingChange,
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
		<ScrollArea className="h-full">
			<div className="py-2">
				<p className="px-3 pb-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
					Data Sources
				</p>
				{templateInfo.sources.map((slot, i) => (
					<div key={slot.namespace}>
						{i > 0 && <Separator className="my-1" />}
						<SourceSlotRow
							slot={slot}
							state={sourcesState[slot.namespace]}
							onFileSelect={onFileSelect}
							onSeparatorChange={onSeparatorChange}
							onEncodingChange={onEncodingChange}
						/>
					</div>
				))}
			</div>
		</ScrollArea>
	);
}
