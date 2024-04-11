use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = PathBuf::from(format!("{manifest_dir}/src/api"));
    let proto = PathBuf::from(format!("{manifest_dir}/proto/measurement.proto"));
    let includes = PathBuf::from(format!("{manifest_dir}/proto"));

    tonic_build::configure()
        .out_dir(out_dir)
        .include_file("mod.rs")
        .compile(&[proto], &[includes])?;

    Ok(())
}
