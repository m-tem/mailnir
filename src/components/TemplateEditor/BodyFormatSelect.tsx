import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";

type BodyFormat = "markdown" | "html" | "text";

interface Props {
	value: BodyFormat | null;
	onChange: (value: BodyFormat) => void;
}

export default function BodyFormatSelect({ value, onChange }: Props) {
	return (
		<div className="space-y-1">
			<p className="text-xs font-medium text-muted-foreground">Body format</p>
			<Select value={value ?? "markdown"} onValueChange={onChange}>
				<SelectTrigger className="h-8 w-40 text-xs">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value="markdown" className="text-xs">
						Markdown (default)
					</SelectItem>
					<SelectItem value="html" className="text-xs">
						HTML
					</SelectItem>
					<SelectItem value="text" className="text-xs">
						Plain Text
					</SelectItem>
				</SelectContent>
			</Select>
		</div>
	);
}
