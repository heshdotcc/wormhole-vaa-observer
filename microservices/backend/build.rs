fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Recompile if any of these files change
    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=build.rs");

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &[
                "proto/spy/v1/spy.proto",
                "proto/gossip/v1/gossip.proto",
                "proto/publicrpc/v1/publicrpc.proto"
            ],
            // Proto directory
            &["proto"] 
        )?;

    Ok(())
}