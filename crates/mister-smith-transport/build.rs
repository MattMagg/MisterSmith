fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = "proto";

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let protoc_include = protoc_bin_vendored::include_path()?;

    std::env::set_var("PROTOC", protoc);

    // Keep `proto/` in the include search path so shared imports such as
    // `import "common.proto";` resolve for all generated modules.
    let include_paths = [
        std::path::PathBuf::from(proto_dir),
        protoc_include,
    ];

    // Compile protobuf message types only (no gRPC service generation).
    // gRPC service code generation is handled in mister-smith-grpc/build.rs.
    prost_build::Config::new().compile_protos(
        &[
            format!("{proto_dir}/common.proto"),
            format!("{proto_dir}/agent_service.proto"),
            format!("{proto_dir}/system_service.proto"),
            format!("{proto_dir}/health_service.proto"),
        ],
        &include_paths,
    )?;

    Ok(())
}
