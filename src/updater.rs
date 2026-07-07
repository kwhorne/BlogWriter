//! Auto-update from GitHub Releases.
//!
//! Checks the `latest.json` manifest attached to the newest GitHub release,
//! and only installs artifacts whose ed25519 signature verifies against the
//! public key embedded below (see `src/bin/sign.rs` and
//! `scripts/make-update-manifest.sh` for the signing side).

use std::path::{Path, PathBuf};
use std::process::Command;

use elyra::updater::{UpdateStatus, Updater};
use elyra::{command, Ctx};
use serde::Serialize;

/// ed25519 public key matching the release-signing private key.
const PUBLIC_KEY_B64: &str = "EcIW2IW4YrB44SfeQg1SeJ1fcL7sgnbNj7E4rHxRVks=";

/// `releases/latest/download/...` always resolves to the newest release.
const MANIFEST_URL: &str =
    "https://github.com/kwhorne/BlogWriter/releases/latest/download/latest.json";

#[derive(Serialize, specta::Type, Clone)]
pub struct UpdateCheck {
    pub available: bool,
    /// Latest version when available, otherwise the running version.
    pub version: String,
    pub notes: String,
    pub current: String,
}

fn make_updater() -> Result<Updater, String> {
    Updater::new(PUBLIC_KEY_B64, env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())
}

fn check_sync() -> Result<UpdateStatus, String> {
    make_updater()?
        .check(MANIFEST_URL, &Updater::current_target())
        .map_err(|e| e.to_string())
}

/// Check GitHub for a newer release.
#[command]
pub async fn check_for_update(_ctx: Ctx) -> Result<UpdateCheck, String> {
    tokio::task::spawn_blocking(|| {
        let current = env!("CARGO_PKG_VERSION").to_string();
        Ok(match check_sync()? {
            UpdateStatus::UpToDate => UpdateCheck {
                available: false,
                version: current.clone(),
                notes: String::new(),
                current,
            },
            UpdateStatus::Available(info) => UpdateCheck {
                available: true,
                version: info.version,
                notes: info.notes.unwrap_or_default(),
                current,
            },
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Download the update (signature-verified), swap it in, and relaunch.
/// On success this call never returns — the app restarts.
#[command]
pub async fn install_update(_ctx: Ctx) -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        let info = match check_sync()? {
            UpdateStatus::UpToDate => return Err("Already up to date.".to_string()),
            UpdateStatus::Available(info) => info,
        };
        let staged = make_updater()?
            .download_verified(&info)
            .map_err(|e| format!("download/verify failed: {e}"))?;
        apply_and_relaunch(&staged)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd} exited with {status}"))
    }
}

fn temp_workdir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("blogwriter-update-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// macOS: the artifact is a zip containing `BlogWriter.app`. Replace the
/// running bundle and relaunch with `open`.
#[cfg(target_os = "macos")]
fn apply_and_relaunch(staged: &Path) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // .../BlogWriter.app/Contents/MacOS/blogwriter -> the .app root
    let bundle = exe
        .ancestors()
        .nth(3)
        .filter(|p| p.extension().is_some_and(|e| e == "app"))
        .ok_or("not running from an .app bundle (dev build?)")?
        .to_path_buf();

    let tmp = temp_workdir()?;
    run("ditto", &["-x", "-k", &staged.to_string_lossy(), &tmp.to_string_lossy()])?;
    let new_app = std::fs::read_dir(&tmp)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|e| e == "app"))
        .ok_or("no .app in update artifact")?;

    // Move the old bundle aside, put the new one in place, roll back on failure.
    let old = bundle.with_extension("app.old");
    let _ = std::fs::remove_dir_all(&old);
    std::fs::rename(&bundle, &old).map_err(|e| format!("move old app: {e}"))?;
    if let Err(e) = run(
        "ditto",
        &[&new_app.to_string_lossy(), &bundle.to_string_lossy()],
    ) {
        let _ = std::fs::rename(&old, &bundle);
        return Err(format!("install failed (rolled back): {e}"));
    }
    let _ = std::fs::remove_dir_all(&old);
    let _ = std::fs::remove_dir_all(&tmp);

    run("open", &["-n", &bundle.to_string_lossy()])?;
    std::process::exit(0);
}

/// Linux: the artifact is a tar.gz containing the `blogwriter` binary.
/// Replace the current executable and re-exec.
#[cfg(target_os = "linux")]
fn apply_and_relaunch(staged: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp = temp_workdir()?;
    run(
        "tar",
        &["-xzf", &staged.to_string_lossy(), "-C", &tmp.to_string_lossy()],
    )?;
    let new_bin = tmp.join("blogwriter");
    if !new_bin.exists() {
        return Err("no blogwriter binary in update artifact".into());
    }

    let old = exe.with_extension("old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&exe, &old).map_err(|e| format!("move old binary: {e}"))?;
    if let Err(e) = std::fs::copy(&new_bin, &exe) {
        let _ = std::fs::rename(&old, &exe);
        return Err(format!("install failed (rolled back): {e}"));
    }
    std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&tmp);

    Command::new(&exe).spawn().map_err(|e| e.to_string())?;
    std::process::exit(0);
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn apply_and_relaunch(_staged: &Path) -> Result<(), String> {
    Err("auto-update is not supported on this platform".into())
}
