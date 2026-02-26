import { PlusIcon, TrashIcon } from "lucide-react";
import { useCallback, useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { SourceSlot } from "@/lib/ipc";

interface Props {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	sources: SourceSlot[];
	onSave: (sources: SourceSlot[]) => void;
}

interface JoinRule {
	localField: string;
	ref: string;
}

interface EditableSource {
	namespace: string;
	isPrimary: boolean;
	isForm: boolean;
	isMany: boolean;
	joinRules: JoinRule[];
}

function toEditable(slot: SourceSlot): EditableSource {
	return {
		namespace: slot.namespace,
		isPrimary: slot.is_primary,
		isForm: slot.is_form,
		isMany: slot.is_many,
		joinRules: Object.entries(slot.join).map(([k, v]) => ({
			localField: k,
			ref: v,
		})),
	};
}

function toSourceSlot(src: EditableSource): SourceSlot {
	const join: Record<string, string> = {};
	for (const rule of src.joinRules) {
		if (rule.localField.trim() && rule.ref.trim()) {
			join[rule.localField.trim()] = rule.ref.trim();
		}
	}
	return {
		namespace: src.namespace,
		is_primary: src.isPrimary,
		has_join: Object.keys(join).length > 0,
		join,
		is_many: src.isMany,
		is_form: src.isForm,
	};
}

function validate(sources: EditableSource[]): string[] {
	const errors: string[] = [];

	const primaries = sources.filter((s) => s.isPrimary);
	if (primaries.length === 0) errors.push("One source must be primary.");
	if (primaries.length > 1) errors.push("Only one source can be primary.");

	const namespaces = sources.map((s) => s.namespace.trim());
	if (namespaces.some((n) => n === ""))
		errors.push("Namespace cannot be empty.");

	const unique = new Set(namespaces);
	if (unique.size !== namespaces.length)
		errors.push("Duplicate namespace names.");

	for (const src of sources) {
		for (const rule of src.joinRules) {
			const ref = rule.ref.trim();
			if (!ref) continue;
			const parts = ref.split(".");
			if (parts.length !== 2 || !parts[0] || !parts[1]) {
				errors.push(
					`Join in "${src.namespace}": "${ref}" must be namespace.field`,
				);
				continue;
			}
			const refNs = parts[0];
			if (refNs === src.namespace.trim()) {
				errors.push(`Join in "${src.namespace}": cannot join to self.`);
			}
			if (!namespaces.includes(refNs)) {
				errors.push(
					`Join in "${src.namespace}": namespace "${refNs}" not found.`,
				);
			}
		}
	}

	return errors;
}

function SourceCard({
	source,
	index,
	onChange,
	onRemove,
	onSetPrimary,
}: {
	source: EditableSource;
	index: number;
	onChange: (index: number, source: EditableSource) => void;
	onRemove: (index: number) => void;
	onSetPrimary: (index: number) => void;
}) {
	const showJoin = !source.isPrimary && !source.isForm;

	return (
		<div className="space-y-3 rounded-md border p-3">
			{/* Header: namespace + remove */}
			<div className="flex items-center gap-2">
				<div className="flex-1">
					<Label className="text-xs text-muted-foreground">Namespace</Label>
					<Input
						value={source.namespace}
						onChange={(e) =>
							onChange(index, { ...source, namespace: e.target.value })
						}
						placeholder="e.g. students"
						className="mt-1 h-7 font-mono text-sm"
					/>
				</div>
				<Button
					size="sm"
					variant="ghost"
					className="mt-5 h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
					onClick={() => onRemove(index)}
				>
					<TrashIcon className="h-3.5 w-3.5" />
				</Button>
			</div>

			{/* Flags row */}
			<div className="flex items-center gap-4">
				<label className="flex items-center gap-1.5 text-xs">
					<input
						type="radio"
						checked={source.isPrimary}
						onChange={() => onSetPrimary(index)}
						className="accent-primary"
					/>
					Primary
				</label>
				<label className="flex items-center gap-1.5 text-xs">
					<input
						type="checkbox"
						checked={source.isForm}
						onChange={(e) =>
							onChange(index, {
								...source,
								isForm: e.target.checked,
								joinRules: e.target.checked ? [] : source.joinRules,
							})
						}
					/>
					Form
				</label>
			</div>

			{/* Join rules */}
			{showJoin && (
				<div className="space-y-2">
					<div className="flex items-center justify-between">
						<span className="text-xs font-medium text-muted-foreground">
							Join Rules
						</span>
						<Button
							size="sm"
							variant="ghost"
							className="h-6 gap-1 px-2 text-xs"
							onClick={() =>
								onChange(index, {
									...source,
									joinRules: [...source.joinRules, { localField: "", ref: "" }],
								})
							}
						>
							<PlusIcon className="h-3 w-3" />
							Add
						</Button>
					</div>
					{source.joinRules.map((rule, ri) => (
						<div
							key={`join-${source.namespace}-${ri}`}
							className="flex items-center gap-1.5"
						>
							<Input
								value={rule.localField}
								onChange={(e) => {
									const rules = [...source.joinRules];
									rules[ri] = { ...rules[ri], localField: e.target.value };
									onChange(index, { ...source, joinRules: rules });
								}}
								placeholder="local_field"
								className="h-7 flex-1 font-mono text-xs"
							/>
							<span className="text-xs text-muted-foreground">&rarr;</span>
							<Input
								value={rule.ref}
								onChange={(e) => {
									const rules = [...source.joinRules];
									rules[ri] = { ...rules[ri], ref: e.target.value };
									onChange(index, { ...source, joinRules: rules });
								}}
								placeholder="namespace.field"
								className="h-7 flex-1 font-mono text-xs"
							/>
							<Button
								size="sm"
								variant="ghost"
								className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
								onClick={() => {
									const rules = source.joinRules.filter((_, j) => j !== ri);
									onChange(index, { ...source, joinRules: rules });
								}}
							>
								<TrashIcon className="h-3 w-3" />
							</Button>
						</div>
					))}
					{source.joinRules.length > 0 && (
						<label className="flex items-center gap-1.5 text-xs">
							<input
								type="checkbox"
								checked={source.isMany}
								onChange={(e) =>
									onChange(index, { ...source, isMany: e.target.checked })
								}
							/>
							Many (1:N)
						</label>
					)}
				</div>
			)}
		</div>
	);
}

export default function SourceConfigDialog({
	open,
	onOpenChange,
	sources,
	onSave,
}: Props) {
	const [editSources, setEditSources] = useState<EditableSource[]>([]);
	const [errors, setErrors] = useState<string[]>([]);

	// Reset working copy when dialog opens.
	const handleOpenChange = useCallback(
		(isOpen: boolean) => {
			if (isOpen) {
				setEditSources(sources.map(toEditable));
				setErrors([]);
			}
			onOpenChange(isOpen);
		},
		[sources, onOpenChange],
	);

	const handleChange = (index: number, updated: EditableSource) => {
		setEditSources((prev) => prev.map((s, i) => (i === index ? updated : s)));
		setErrors([]);
	};

	const handleSetPrimary = (index: number) => {
		setEditSources((prev) =>
			prev.map((s, i) => ({ ...s, isPrimary: i === index })),
		);
		setErrors([]);
	};

	const handleRemove = (index: number) => {
		setEditSources((prev) => prev.filter((_, i) => i !== index));
		setErrors([]);
	};

	const handleAdd = () => {
		setEditSources((prev) => [
			...prev,
			{
				namespace: "",
				isPrimary: prev.length === 0,
				isForm: false,
				isMany: false,
				joinRules: [],
			},
		]);
		setErrors([]);
	};

	const handleSave = () => {
		const validationErrors = validate(editSources);
		if (validationErrors.length > 0) {
			setErrors(validationErrors);
			return;
		}
		onSave(editSources.map(toSourceSlot));
		onOpenChange(false);
	};

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogContent className="max-w-lg">
				<DialogHeader>
					<DialogTitle>Configure Sources</DialogTitle>
				</DialogHeader>
				<ScrollArea className="max-h-[60vh]">
					<div className="space-y-3 pr-4">
						{editSources.map((src, i) => (
							<SourceCard
								key={`source-${i}`}
								source={src}
								index={i}
								onChange={handleChange}
								onRemove={handleRemove}
								onSetPrimary={handleSetPrimary}
							/>
						))}
						{editSources.length === 0 && (
							<p className="py-4 text-center text-sm text-muted-foreground">
								No sources. Add one to get started.
							</p>
						)}
					</div>
				</ScrollArea>
				{errors.length > 0 && (
					<>
						<Separator />
						<div className="space-y-1">
							{errors.map((err) => (
								<p key={err} className="text-xs text-destructive">
									{err}
								</p>
							))}
						</div>
					</>
				)}
				<Separator />
				<div className="flex items-center justify-between">
					<Button
						size="sm"
						variant="outline"
						className="gap-1"
						onClick={handleAdd}
					>
						<PlusIcon className="h-3.5 w-3.5" />
						Add Source
					</Button>
					<div className="flex items-center gap-2">
						<Button
							size="sm"
							variant="ghost"
							onClick={() => onOpenChange(false)}
						>
							Cancel
						</Button>
						<Button size="sm" onClick={handleSave}>
							Save
						</Button>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
}
