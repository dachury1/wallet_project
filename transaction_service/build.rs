fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This sets the PROTOC environment variable so tonic-build uses the vendored binary
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["proto/wallet.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    Ok(())
}
