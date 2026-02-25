fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[
                "proto/heath.proto",
                "proto/auth.proto",
                "proto/protected.proto",
            ],
            &["proto"],
        )?;

    println!("cargo:rerun-if-changed=proto/heath.proto");
    println!("cargo:rerun-if-changed=proto/auth.proto");
    println!("cargo:rerun-if-changed=proto/protected.proto");
    Ok(())
}
