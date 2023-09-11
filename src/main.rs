use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context};
use cargo_metadata::MetadataCommand;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    /// Allow operation even with a dirty git working directory.
    #[arg(long)]
    allow_dirty: bool,
}

fn git_dirty_cmd() -> anyhow::Result<bool> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .stderr(Stdio::inherit())
        .output()
        .context("failed to run git")?;

    if !output.status.success() {
        bail!("git exited with status {}", output.status);
    }

    Ok(!output.stdout.is_empty())
}

fn git_dirty() -> bool {
    git_dirty_cmd()
        .unwrap_or_else(|e| {
            eprintln!("failed checking git working dir: {e}");
            true
        })
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !args.allow_dirty && git_dirty() {
        bail!("\
            Running this command with a dirty git working directory is unwise. \
            Either commit pending changes (so you can see what changes this program makes) \
            or run again with the --allow-dirty flag.");
    }

    let meta = MetadataCommand::new()
        .no_deps()
        .other_options(vec!["--offline".to_owned()])
        .exec()
        .context("cargo metadata")?;

    let ws_root = meta.workspace_root;

    // Can also get this by itself by running 'cargo locate-project --workspace'
    let ws_toml = PathBuf::from(ws_root).join("Cargo.toml");
    let ws_toml_renamed = ws_toml.with_file_name("_Cargo_sync_temp.toml");
    let ws_lock = ws_toml.with_file_name("Cargo.lock");

    println!("workspace root toml: {ws_toml:?}");

    let ws_members = meta.workspace_members;
    println!("workspace members: {ws_members:#?}");

    if ws_members.len() < 2 {
        bail!("no point in running this program without multiple workspace members");
    }

    // rename workspace Cargo.toml to something else
    fs::rename(&ws_toml, &ws_toml_renamed)
        .context("failed to rename workspace root Cargo.toml")?;

    // foreach workspace member:
    for pkgid in &ws_members {
        let pkg = meta.packages.iter().find(|p| &p.id == pkgid).expect("unable to find package");
        let pkg_dir = pkg.manifest_path.parent().unwrap();
        eprintln!("{pkgid}: {pkg_dir}");

        env::set_current_dir(&pkg_dir).context("failed to chdir into workspace member")?;

        fs::copy(&ws_lock, pkg_dir.join("Cargo.lock"))
            .context("failed to copy root lockfile to workspace member")?;

        // run some cargo command in member context to fix the lock file
        MetadataCommand::new()
            .other_options(vec!["--offline".to_owned()])
            .exec()
            .with_context(|| format!("failed to run cargo metadata in workspace member {}", pkg.name))?;
    }

    // rename workspace Cargo.toml back
    fs::rename(&ws_toml_renamed, &ws_toml)
        .context("failed to rename back workspace root Cargo.toml")?;

    Ok(())
}
