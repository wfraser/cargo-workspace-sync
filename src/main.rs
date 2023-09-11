use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context};
use clap::Parser;
use serde_json::Value;

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

    Ok(output.stdout != b"\n")
}

fn git_dirty() -> bool {
    git_dirty_cmd()
        .unwrap_or_else(|e| {
            eprintln!("failed checking git working dir: {e}");
            true
        })
}

fn cmd_json(cmd: &mut Command) -> anyhow::Result<serde_json::Value> {
    let output = cmd.stderr(Stdio::inherit()).output().context("failed to run command")?;

    if !output.status.success() {
        bail!("command exited with status {}", output.status);
    }

    let json = String::from_utf8_lossy(&output.stdout)
        .parse::<serde_json::Value>()
        .context("invalid json")?;

    Ok(json)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !args.allow_dirty && git_dirty() {
        bail!("\
            Running this command with a dirty git working directory is unwise. \
            Either commit pending changes (so you can see what changes this program makes) \
            or run again with the --allow-dirty flag.");
    }

    let meta = match cmd_json(
        Command::new("cargo")
            .args([
                "metadata",
                "--format-version=1",
                "--offline",
                "--no-deps",
            ])
    )
        .context("cargo metadata")?
    {
        Value::Object(o) => o,
        other => bail!("expected a json object, not {}", other),
    };

    let ws_root = meta.get("workspace_root")
        .ok_or_else(|| anyhow!("expected a 'workspace_root' field"))?
        .as_str()
        .ok_or_else(|| anyhow!("expected 'workspace_root' to be a string"))?;

    // Can also get this by itself by running 'cargo locate-project --workspace'
    let ws_toml = PathBuf::from(ws_root).join("Cargo.toml");

    println!("ws toml: {ws_toml:?}");

    let ws_members = meta
        .get("workspace_members")
        .ok_or_else(|| anyhow!("expected a 'workspace_members' field"))?
        .as_array()
        .ok_or_else(|| anyhow!("expected 'workspace_members' to be an array"))?;

    println!("ws members: {ws_members:#?}");

    // rename workspace Cargo.toml to something else

    // foreach workspace member:
    //      copy root Cargo.lock into workspace
    //      run some cargo command in member context to fix the lock file

    // rename workspace Cargo.toml back

    Ok(())
}
