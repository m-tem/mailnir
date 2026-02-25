import { AlertTriangleIcon } from "lucide-react";

interface Props {
	issues: string[];
}

export default function ValidationErrors({ issues }: Props) {
	if (issues.length === 0) return null;

	return (
		<div className="border-t bg-amber-50 px-3 py-2 dark:bg-amber-950/30">
			<div className="mb-1 flex items-center gap-1.5">
				<AlertTriangleIcon className="size-3.5 text-amber-600 dark:text-amber-400" />
				<span className="text-xs font-medium text-amber-800 dark:text-amber-300">
					Validation issues
				</span>
			</div>
			<ul className="space-y-0.5 pl-5">
				{issues.map((issue) => (
					<li
						key={issue}
						className="list-disc text-xs text-amber-700 dark:text-amber-400"
					>
						{issue}
					</li>
				))}
			</ul>
		</div>
	);
}
