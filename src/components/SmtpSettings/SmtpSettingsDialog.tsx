import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { SmtpProfile } from "@/lib/ipc";
import {
	deleteSmtpCredential,
	storeSmtpCredential,
	testSmtpConnection,
} from "@/lib/ipc";
import SmtpProfileForm from "./SmtpProfileForm";

interface Props {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	profiles: SmtpProfile[];
	onSave: (profiles: SmtpProfile[]) => Promise<void>;
}

type View =
	| { kind: "list" }
	| { kind: "add" }
	| { kind: "edit"; profile: SmtpProfile };

export default function SmtpSettingsDialog({
	open,
	onOpenChange,
	profiles,
	onSave,
}: Props) {
	const [view, setView] = useState<View>({ kind: "list" });
	const [testingName, setTestingName] = useState<string | null>(null);
	const [testResult, setTestResult] = useState<Record<string, string>>({});
	const [testUsername, setTestUsername] = useState("");
	const [testPassword, setTestPassword] = useState("");
	const [testTarget, setTestTarget] = useState<SmtpProfile | null>(null);

	const handleSaveProfile = async (
		profile: SmtpProfile,
		username: string,
		password: string,
	) => {
		const existing = profiles.find((p) => p.name === profile.name);
		const updated = existing
			? profiles.map((p) => (p.name === profile.name ? profile : p))
			: [...profiles, profile];
		await onSave(updated);
		if (username && password) {
			await storeSmtpCredential(profile.name, username, password);
		}
		setView({ kind: "list" });
	};

	const handleDelete = async (name: string) => {
		if (!confirm(`Delete profile "${name}"?`)) return;
		await deleteSmtpCredential(name).catch(() => {});
		await onSave(profiles.filter((p) => p.name !== name));
	};

	const handleTestConnection = async (profile: SmtpProfile) => {
		if (testTarget?.name === profile.name) {
			// Submit test
			setTestingName(profile.name);
			try {
				await testSmtpConnection(profile, testUsername, testPassword);
				setTestResult((r) => ({
					...r,
					[profile.name]: "✓ Connection successful",
				}));
			} catch (err) {
				setTestResult((r) => ({ ...r, [profile.name]: `✕ ${String(err)}` }));
			} finally {
				setTestingName(null);
				setTestTarget(null);
				setTestUsername("");
				setTestPassword("");
			}
		} else {
			// Ask for credentials first
			setTestTarget(profile);
			setTestUsername("");
			setTestPassword("");
		}
	};

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-w-lg">
				<DialogHeader>
					<DialogTitle>SMTP Profiles</DialogTitle>
				</DialogHeader>

				{view.kind === "list" && (
					<div className="space-y-3">
						<ScrollArea className="max-h-72">
							{profiles.length === 0 ? (
								<p className="py-4 text-center text-sm text-muted-foreground">
									No profiles yet
								</p>
							) : (
								<div className="space-y-1">
									{profiles.map((p, i) => (
										<div key={p.name}>
											{i > 0 && <Separator className="my-1" />}
											<div className="rounded-md p-2">
												<div className="flex items-start justify-between gap-2">
													<div className="min-w-0">
														<p className="font-medium">{p.name}</p>
														<p className="text-xs text-muted-foreground">
															{p.host}:{p.port} · {p.encryption} · from {p.from}
														</p>
														{testResult[p.name] && (
															<p
																className={`text-xs ${
																	testResult[p.name].startsWith("✓")
																		? "text-green-600"
																		: "text-destructive"
																}`}
															>
																{testResult[p.name]}
															</p>
														)}
														{testTarget?.name === p.name && (
															<div className="mt-1 space-y-1">
																<input
																	className="w-full rounded border px-1 py-0.5 text-xs"
																	placeholder="Username"
																	value={testUsername}
																	onChange={(e) =>
																		setTestUsername(e.target.value)
																	}
																/>
																<input
																	type="password"
																	className="w-full rounded border px-1 py-0.5 text-xs"
																	placeholder="Password"
																	value={testPassword}
																	onChange={(e) =>
																		setTestPassword(e.target.value)
																	}
																/>
															</div>
														)}
													</div>
													<div className="flex shrink-0 gap-1">
														<Button
															size="sm"
															variant="outline"
															className="h-6 px-2 text-xs"
															onClick={() => handleTestConnection(p)}
															disabled={testingName === p.name}
														>
															{testingName === p.name
																? "Testing…"
																: testTarget?.name === p.name
																	? "Run"
																	: "Test"}
														</Button>
														<Button
															size="sm"
															variant="ghost"
															className="h-6 px-2 text-xs"
															onClick={() =>
																setView({ kind: "edit", profile: p })
															}
														>
															Edit
														</Button>
														<Button
															size="sm"
															variant="ghost"
															className="h-6 px-2 text-xs text-destructive hover:text-destructive"
															onClick={() => handleDelete(p.name)}
														>
															Delete
														</Button>
													</div>
												</div>
											</div>
										</div>
									))}
								</div>
							)}
						</ScrollArea>
						<Button
							className="w-full"
							variant="outline"
							onClick={() => setView({ kind: "add" })}
						>
							+ New profile
						</Button>
					</div>
				)}

				{view.kind === "add" && (
					<SmtpProfileForm
						onSave={handleSaveProfile}
						onCancel={() => setView({ kind: "list" })}
					/>
				)}

				{view.kind === "edit" && (
					<SmtpProfileForm
						initial={view.profile}
						onSave={handleSaveProfile}
						onCancel={() => setView({ kind: "list" })}
					/>
				)}
			</DialogContent>
		</Dialog>
	);
}
