fn main() {
    let proto_files = vec![
        "proto/trading_service.proto",
        "proto/vertex_products.proto",
        "proto/vertex_query.proto",
        "proto/vertex_execute.proto",
        "proto/vertex_symbols.proto",
    ];

    tonic_build::configure()
        .build_client(true) // generate client code for gRPC-Web compatibility
        .build_server(true)
        .type_attribute(
            ".", // Use the package name as the namespace
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(".", "#[serde(rename_all = \"snake_case\")]")
        .compile(
            &proto_files,
            &["proto"], // Adjust this path as per your project structurectory here
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos: {}", e));
}
