fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = "proto";

    // Compile protobuf message types only (no gRPC service generation).
    // gRPC service code generation is handled in mister-smith-grpc/build.rs.
    prost_build::Config::new()
        .compile_protos(
            &[
                format!("{proto_dir}/common.proto"),
                format!("{proto_dir}/agent_service.proto"),
                format!("{proto_dir}/system_service.proto"),
            ],
            &[proto_dir],
        )?;

    Ok(())
}
