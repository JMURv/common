#[cfg(feature = "contracts")]
use std::{fs, path::Path};

#[cfg(feature = "contracts")]
use anyhow::{Context, Result};

#[cfg(feature = "contracts")]
fn main() -> Result<()> {
    const CONTRACT_ROOT: &str = "contracts";
    let mut contracts = Vec::new();
    collect_proto_files(Path::new(CONTRACT_ROOT), &mut contracts)?;
    contracts.sort();

    println!("cargo:rerun-if-changed={CONTRACT_ROOT}");
    tonic_prost_build::configure()
        .bytes(".benito.realtime.internal.v1.FanoutEnvelope.server_frame")
        .compile_protos(&contracts, &[Path::new(CONTRACT_ROOT).to_path_buf()])
        .context("compile Benito protobuf contracts")?;
    Ok(())
}

#[cfg(feature = "contracts")]
fn collect_proto_files(directory: &Path, output: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory).with_context(|| format!("read proto directory {}", directory.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_proto_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "proto") {
            output.push(path);
        }
    }
    Ok(())
}

#[cfg(not(feature = "contracts"))]
fn main() {}
