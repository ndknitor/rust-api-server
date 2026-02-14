use std::fs;
use std::path::{Path, PathBuf};

fn collect_proto_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_proto_files(&path, out)?;
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "proto") {
            out.push(path);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = Path::new("proto");
    println!("cargo:rerun-if-changed={}", proto_root.display());

    let mut proto_files = Vec::new();
    collect_proto_files(proto_root, &mut proto_files)?;
    proto_files.sort();

    if proto_files.is_empty() {
        return Err("no .proto files found under ./proto".into());
    }

    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&proto_files, &[proto_root])?;

    Ok(())
}
