fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/proto")
        .compile(&["proto/internalfiles.proto"], &["."])
        .expect("failed to compile protos");
    Ok(())
}
