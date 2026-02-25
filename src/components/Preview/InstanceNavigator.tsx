import {
	AlertTriangleIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import type { PreviewEntryStatus } from "@/lib/ipc";

interface Props {
	entryCount: number;
	currentIndex: number;
	entries: PreviewEntryStatus[];
	onNavigate: (index: number) => void;
}

export default function InstanceNavigator({
	entryCount,
	currentIndex,
	entries,
	onNavigate,
}: Props) {
	const currentEntry = entries[currentIndex];
	const hasError = currentEntry != null && !currentEntry.is_valid;

	return (
		<div className="flex items-center gap-1 border-b px-3 py-1.5">
			<Button
				variant="ghost"
				size="sm"
				className="h-6 w-6 p-0"
				disabled={currentIndex === 0}
				onClick={() => onNavigate(currentIndex - 1)}
			>
				<ChevronLeftIcon className="size-4" />
			</Button>
			<span className="flex items-center gap-1.5 text-xs tabular-nums">
				{currentIndex + 1} of {entryCount}
				{hasError && <AlertTriangleIcon className="size-3.5 text-amber-500" />}
			</span>
			<Button
				variant="ghost"
				size="sm"
				className="h-6 w-6 p-0"
				disabled={currentIndex >= entryCount - 1}
				onClick={() => onNavigate(currentIndex + 1)}
			>
				<ChevronRightIcon className="size-4" />
			</Button>
		</div>
	);
}
