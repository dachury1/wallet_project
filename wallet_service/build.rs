fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This sets the PROTOC environment variable so tonic-build uses the vendored binary
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    tonic_build::compile_protos("proto/wallet.proto")?;
    Ok(())
}
