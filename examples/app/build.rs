use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    app_forge_kit_grpc_common::proto::configure()
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .emit_rerun_if_changed(true)
        .emit_package(true)
        .compile_well_known_types(true)
        .with_extended_rust_types(true)
        .file_descriptor_set_path(out_dir.join("echo.bin"))
        .compile_protos(&["echo.proto"], &["."])?;

    Ok(())
}
