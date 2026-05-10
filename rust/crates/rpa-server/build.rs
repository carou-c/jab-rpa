fn main() {
    // 1. Tell cargo to rerun if build.rs or proto change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../../proto/jab.proto");

    // 2. Compile proto (NEW)
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["../../../proto/jab.proto"], &["../../../proto/"])
        .expect("Couldn't build protos!");
}
