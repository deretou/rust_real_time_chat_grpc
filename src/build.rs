fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .compile(&["src/protos/chat.proto"], &["protos"])?;
    Ok(())
}
