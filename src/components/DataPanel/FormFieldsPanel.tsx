import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface Props {
	fields: string[];
	values: Record<string, string>;
	onValueChange: (field: string, value: string) => void;
}

export default function FormFieldsPanel({
	fields,
	values,
	onValueChange,
}: Props) {
	if (fields.length === 0) {
		return (
			<div className="mt-2 rounded-md border bg-muted/30 p-2">
				<p className="text-xs text-muted-foreground">
					No fields detected. Use {"{{namespace.field}}"} in your template.
				</p>
			</div>
		);
	}

	return (
		<div className="mt-2 space-y-2 rounded-md border bg-muted/30 p-2">
			{fields.map((field) => (
				<div key={field} className="space-y-0.5">
					<Label className="text-xs text-muted-foreground">{field}</Label>
					<Input
						className="h-7 text-xs"
						value={values[field] ?? ""}
						onChange={(e) => onValueChange(field, e.target.value)}
						placeholder={field}
					/>
				</div>
			))}
		</div>
	);
}
