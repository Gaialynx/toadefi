fn main() {
    tonic_build::configure()
        .build_server(true)
        .type_attribute(
            ".", // Use the package name as the namespace
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(".", "#[serde(rename_all = \"snake_case\")]")
        .compile(
            &[
                "proto/vertex/trading_service.proto",
                "proto/vertex/vertex_query.proto",
            ],
            &["proto/", "proto/google/api/"], // Adjust this path as per your project structurectory here
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos: {}", e));
}
