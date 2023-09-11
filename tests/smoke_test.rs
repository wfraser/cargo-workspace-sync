use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio, ExitStatus};

use anyhow::anyhow;
use tempfile::tempdir;

trait CmdStatus {
    fn success(self) -> anyhow::Result<()>;
}

impl CmdStatus for Result<ExitStatus, io::Error> {
    fn success(self) -> anyhow::Result<()> {
        match self {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(anyhow!("process failed: {status}")),
            Err(e) => Err(anyhow!("failed to run process: {e}")),
        }
    }
}

#[test]
fn smoke_test() {
    let test_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-workspace");

    let tmpdir = tempdir().unwrap();
    let path = tmpdir.path().join("test-workspace");

    // note: it's not safe to set the cwd because tests are run multi-threaded and cwd is
    // per-process, not per-thread. So always pass working dir to subcommands explicitly.

    Command::new("cp")
        .arg("-pruv")
        .arg(test_src)
        .arg(tmpdir.path())
        .status()
        .success()
        .expect("cp test fixtures");

    Command::new("git")
        .arg("init")
        .current_dir(&path)
        .status()
        .success()
        .expect("git init");

    Command::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .status()
        .success()
        .expect("git add");

    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(&path)
        .status()
        .success()
        .expect("git commit");

    Command::new("cargo")
        .args(["update", "--package=env_logger@0.9.0", "--precise=0.9.3"])
        .current_dir(&path)
        .status()
        .success()
        .expect("cargo update");

    Command::new("git")
        .args(["commit", "-m", "update", "Cargo.lock"])
        .current_dir(&path)
        .status()
        .success()
        .expect("git commit");

    Command::new(env!("CARGO_BIN_EXE_cargo-sync"))
        .current_dir(&path)
        .status()
        .success()
        .expect("cargo-sync");

    let out = Command::new("git")
        .arg("diff")
        .current_dir(&path)
        .stderr(Stdio::inherit())
        .output()
        .expect("git diff");

    if !out.status.success() {
        panic!("git diff exit status: {}", out.status);
    }

    let out = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");

    assert_eq!(out, concat!("\
diff --git a/a/Cargo.lock b/a/Cargo.lock
index cb9633f..35912eb 100644
--- a/a/Cargo.lock
+++ b/a/Cargo.lock
@@ -31,9 +31,9 @@ dependencies = [
 ", /* single space, don't trim me bro */ "
 [[package]]
 name = \"env_logger\"
-version = \"0.9.0\"
+version = \"0.9.3\"
 source = \"registry+https://github.com/rust-lang/crates.io-index\"
-checksum = \"0b2cf0344971ee6c64c31be0d530793fba457d322dfec2810c453d0ef228f9c3\"
+checksum = \"a12e6657c4c97ebab115a42dcee77225f7f482cdd841cf7088c657a42e9e00e7\"
 dependencies = [
  \"atty\",
  \"humantime\",
"));

}