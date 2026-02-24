import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import type { SmtpProfile } from "@/lib/ipc";

interface Props {
	initial?: SmtpProfile;
	onSave: (
		profile: SmtpProfile,
		username: string,
		password: string,
	) => Promise<void>;
	onCancel: () => void;
}

const emptyProfile: SmtpProfile = {
	name: "",
	host: "",
	port: 587,
	encryption: "start_tls",
	from: "",
	parallelism: 1,
};

export default function SmtpProfileForm({ initial, onSave, onCancel }: Props) {
	const [profile, setProfile] = useState<SmtpProfile>(initial ?? emptyProfile);
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const [saving, setSaving] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const set = <K extends keyof SmtpProfile>(key: K, value: SmtpProfile[K]) =>
		setProfile((p) => ({ ...p, [key]: value }));

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setSaving(true);
		setError(null);
		try {
			await onSave(profile, username, password);
		} catch (err) {
			setError(String(err));
			setSaving(false);
		}
	};

	return (
		<form onSubmit={handleSubmit} className="space-y-4">
			<div className="grid grid-cols-2 gap-3">
				<div className="col-span-2 space-y-1">
					<Label htmlFor="profile-name">Profile name</Label>
					<Input
						id="profile-name"
						value={profile.name}
						onChange={(e) => set("name", e.target.value)}
						placeholder="e.g. Work SMTP"
						required
						disabled={!!initial}
					/>
				</div>

				<div className="col-span-2 space-y-1">
					<Label htmlFor="smtp-host">Host</Label>
					<Input
						id="smtp-host"
						value={profile.host}
						onChange={(e) => set("host", e.target.value)}
						placeholder="smtp.example.com"
						required
					/>
				</div>

				<div className="space-y-1">
					<Label htmlFor="smtp-port">Port</Label>
					<Input
						id="smtp-port"
						type="number"
						min={1}
						max={65535}
						value={profile.port}
						onChange={(e) => set("port", parseInt(e.target.value, 10))}
						required
					/>
				</div>

				<div className="space-y-1">
					<Label>Encryption</Label>
					<Select
						value={profile.encryption}
						onValueChange={(v) =>
							set("encryption", v as SmtpProfile["encryption"])
						}
					>
						<SelectTrigger>
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="tls">TLS</SelectItem>
							<SelectItem value="start_tls">STARTTLS</SelectItem>
							<SelectItem value="none">None</SelectItem>
						</SelectContent>
					</Select>
				</div>

				<div className="col-span-2 space-y-1">
					<Label htmlFor="smtp-from">From address</Label>
					<Input
						id="smtp-from"
						type="email"
						value={profile.from}
						onChange={(e) => set("from", e.target.value)}
						placeholder="sender@example.com"
						required
					/>
				</div>

				<div className="space-y-1">
					<Label htmlFor="smtp-parallelism">Parallelism</Label>
					<Input
						id="smtp-parallelism"
						type="number"
						min={1}
						max={20}
						value={profile.parallelism}
						onChange={(e) => set("parallelism", parseInt(e.target.value, 10))}
					/>
				</div>
			</div>

			<Separator />

			<div className="space-y-3">
				<p className="text-sm font-medium">Credentials</p>
				<div className="space-y-1">
					<Label htmlFor="smtp-username">Username</Label>
					<Input
						id="smtp-username"
						value={username}
						onChange={(e) => setUsername(e.target.value)}
						placeholder="username or email"
					/>
				</div>
				<div className="space-y-1">
					<Label htmlFor="smtp-password">Password</Label>
					<Input
						id="smtp-password"
						type="password"
						value={password}
						onChange={(e) => setPassword(e.target.value)}
						placeholder="stored in OS keychain"
					/>
				</div>
				<p className="text-xs text-muted-foreground">
					Credentials are stored in the OS keychain, not in the profile file.
					Leave blank to keep existing credentials.
				</p>
			</div>

			{error && <p className="text-sm text-destructive">{error}</p>}

			<div className="flex justify-end gap-2">
				<Button type="button" variant="outline" onClick={onCancel}>
					Cancel
				</Button>
				<Button type="submit" disabled={saving}>
					{saving ? "Saving…" : "Save"}
				</Button>
			</div>
		</form>
	);
}
