use anyhow::{bail, Result};
use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};

pub const WASM32_WASI_TARGET: &str = "wasm32-wasi";

pub fn install_wasm32_wasi() -> Result<()> {
    log::info!("Installing {WASM32_WASI_TARGET} target");
    let sysroot = get_sysroot()?;
    if sysroot.join("lib/rustlib/wasm32-wasi").exists() {
        return Ok(());
    }

    if env::var_os("RUSTUP_TOOLCHAIN").is_none() {
        bail!(
            "failed to find the `wasm32-wasi` target \
             and `rustup` is not available. If you're using rustup \
             make sure that it's correctly installed; if not, make sure to \
             install the `wasm32-wasi` target before using this command"
        );
    }

    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg(WASM32_WASI_TARGET)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        bail!("failed to install the `wasm32-wasi` target");
    }

    Ok(())
}

fn get_sysroot() -> Result<PathBuf> {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()?;

    if !output.status.success() {
        bail!(
            "failed to execute `rustc --print sysroot`, \
                 command exited with error: {output}",
            output = String::from_utf8_lossy(&output.stderr)
        );
    }

    let sysroot = PathBuf::from(String::from_utf8(output.stdout)?.trim());

    Ok(sysroot)
}
