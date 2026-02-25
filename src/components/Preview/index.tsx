import { LoaderCircleIcon } from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { PreviewRenderedEmail, PreviewValidation } from "@/lib/ipc";
import HtmlPreview from "./HtmlPreview";
import InstanceNavigator from "./InstanceNavigator";
import MetadataPanel from "./MetadataPanel";
import TextPreview from "./TextPreview";
import ValidationErrors from "./ValidationErrors";

interface Props {
	validation: PreviewValidation | null;
	currentIndex: number;
	rendered: PreviewRenderedEmail | null;
	loading: boolean;
	error: string | null;
	onNavigate: (index: number) => void;
}

export default function Preview({
	validation,
	currentIndex,
	rendered,
	loading,
	error,
	onNavigate,
}: Props) {
	// No preview data yet — placeholder
	if (!validation && !loading && !error) {
		return (
			<div className="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
				<p className="text-sm font-medium text-muted-foreground">Preview</p>
				<p className="text-xs text-muted-foreground">
					Load a template and data sources to see a preview
				</p>
			</div>
		);
	}

	const currentEntry = validation?.entries[currentIndex];
	const currentIssues = currentEntry?.issues ?? [];

	return (
		<div className="flex h-full flex-col">
			{/* Loading overlay */}
			{loading && (
				<div className="flex items-center gap-2 border-b px-3 py-1.5">
					<LoaderCircleIcon className="size-3.5 animate-spin text-muted-foreground" />
					<span className="text-xs text-muted-foreground">Rendering...</span>
				</div>
			)}

			{/* Error display */}
			{error && (
				<div className="border-b px-3 py-2">
					<p className="text-xs text-destructive">{error}</p>
				</div>
			)}

			{/* Instance navigator */}
			{validation && validation.entry_count > 0 && (
				<InstanceNavigator
					entryCount={validation.entry_count}
					currentIndex={currentIndex}
					entries={validation.entries}
					onNavigate={onNavigate}
				/>
			)}

			{/* Empty state */}
			{validation && validation.entry_count === 0 && (
				<div className="flex flex-1 items-center justify-center p-4">
					<p className="text-xs text-muted-foreground">No entries to preview</p>
				</div>
			)}

			{/* Metadata */}
			<MetadataPanel rendered={rendered} />

			{/* HTML / Text tabs */}
			{rendered && (
				<Tabs defaultValue="html" className="min-h-0 flex-1">
					<TabsList className="mx-3 mt-1">
						<TabsTrigger value="html">HTML</TabsTrigger>
						<TabsTrigger value="text">Text</TabsTrigger>
					</TabsList>
					<TabsContent value="html" className="relative min-h-0">
						<HtmlPreview html={rendered.html_body} />
					</TabsContent>
					<TabsContent value="text" className="min-h-0 overflow-y-auto">
						<TextPreview text={rendered.text_body} />
					</TabsContent>
				</Tabs>
			)}

			{/* Validation errors */}
			<ValidationErrors issues={currentIssues} />
		</div>
	);
}
