use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context};
use miden_diagnostics::Verbosity;
use midenc_session::{
    InputFile, OutputFile, OutputType, OutputTypeSpec, OutputTypes, ProjectType, Session, TargetEnv,
};

pub fn build_masm(
    wasm_file_path: &Path,
    output_folder: &Path,
    is_bin: bool,
) -> anyhow::Result<PathBuf> {
    let project_type = if is_bin {
        ProjectType::Program
    } else {
        ProjectType::Library
    };

    if !output_folder.exists() {
        bail!(
            "MASM output folder '{}' does not exist.",
            output_folder.to_str().unwrap()
        );
    }
    log::debug!(
        "Compiling '{}' Wasm to '{}' directory with midenc ...",
        wasm_file_path.to_str().unwrap(),
        &output_folder.to_str().unwrap()
    );
    let input = InputFile::from_path(wasm_file_path).context("Invalid input file")?;
    let output_file_folder = OutputFile::Real(output_folder.to_path_buf());
    let output_type = OutputType::Masm;
    let output_types = OutputTypes::new(vec![OutputTypeSpec {
        output_type,
        path: Some(output_file_folder.clone()),
    }]);
    let cwd = std::env::current_dir().context("Failed to get current working directory")?;
    let options = midenc_session::Options::new(cwd)
        // .with_color(color)
        .with_verbosity(Verbosity::Debug)
        // .with_warnings(self.warn)
        .with_output_types(output_types);
    let target = TargetEnv::default();
    let session = Arc::new(
        Session::new(
            target,
            input,
            Some(output_folder.to_path_buf()),
            None,
            None,
            options,
            None,
        )
        .with_project_type(project_type),
    );
    midenc_compile::compile(session.clone()).context("Wasm to MASM compilation failed!")?;
    let mut output_path = output_folder.join(wasm_file_path.file_stem().unwrap());
    output_path.set_extension(output_type.extension());
    Ok(output_path)
}
