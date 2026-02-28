import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import type { SmtpProfile } from "@/lib/ipc";
import { getVersionInfo } from "@/lib/ipc";

interface Props {
	profiles: SmtpProfile[];
	selectedProfileName: string | null;
	onProfileChange: (name: string) => void;
	onSmtpSettings: () => void;
	allSourcesLoaded: boolean;
	previewVisible: boolean;
	onTogglePreview: () => void;
	onSend: () => void;
}

export default function StatusBar({
	profiles,
	selectedProfileName,
	onProfileChange,
	onSmtpSettings,
	allSourcesLoaded,
	previewVisible,
	onTogglePreview,
	onSend,
}: Props) {
	const [version, setVersion] = useState("");
	useEffect(() => {
		getVersionInfo()
			.then(setVersion)
			.catch(() => {});
	}, []);

	return (
		<div className="flex items-center gap-2 border-t bg-background px-4 py-2">
			{version && (
				<span className="text-[10px] text-muted-foreground select-all">
					{version}
				</span>
			)}
			<Select
				value={selectedProfileName ?? ""}
				onValueChange={onProfileChange}
				disabled={profiles.length === 0}
			>
				<SelectTrigger className="h-7 w-44 text-xs">
					<SelectValue placeholder="No SMTP profile" />
				</SelectTrigger>
				<SelectContent>
					{profiles.map((p) => (
						<SelectItem key={p.name} value={p.name} className="text-xs">
							{p.name}
						</SelectItem>
					))}
				</SelectContent>
			</Select>

			<Button
				size="sm"
				variant="ghost"
				className="h-7 px-2 text-xs"
				onClick={onSmtpSettings}
			>
				SMTP Settings
			</Button>

			<div className="flex-1" />

			<Button
				size="sm"
				variant={previewVisible ? "default" : "outline"}
				className="h-7 text-xs"
				onClick={onTogglePreview}
			>
				Preview
			</Button>
			<Button
				size="sm"
				className="h-7 text-xs"
				disabled={!allSourcesLoaded || !selectedProfileName}
				onClick={onSend}
			>
				Send
			</Button>
		</div>
	);
}
