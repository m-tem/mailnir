use std::process::Command;

fn main() {
    // Capture git version string for display. Requires git to be installed.
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty=*"])
        .output()
        .expect("git must be installed to build mailnir");

    assert!(
        output.status.success(),
        "git describe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("cargo:rustc-env=GIT_VERSION={version}");

    // Re-run when HEAD or the index change (.git is at the repo root).
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/index");

    if std::env::var("CARGO_FEATURE_TAURI_BACKEND").is_ok() {
        tauri_build::build();
    }
}
