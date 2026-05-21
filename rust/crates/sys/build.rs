use std::env;
use std::path::PathBuf;
use std::process::Command;

// const MINGW_SYSROOT: &str = "/usr/i686-w64-mingw32";

fn main() {
    let arch_flag = {
        let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("Couldn't get target architecture");
        if arch == "x86" {
            "ACCESSBRIDGE_ARCH_32"
        } else if arch == "x86_64" {
            "ACCESSBRIDGE_ARCH_64"
        } else {
            panic!("unsupported architecture")
        }
    };

    let openjdk_dir = selected_openjdk_dir();

    // 1. Compile the C code
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .prefer_clang_cl_over_msvc(true)
        .warnings(false)
        .define(arch_flag, None)
        .include(format!("{}/bridge", openjdk_dir)) // for headers
        .include(format!("{}/jni", openjdk_dir)) // for headers
        .include(format!("{}/jni/linux", openjdk_dir)) // for headers
        .file(format!("{}/bridge/AccessBridgeCalls.c", openjdk_dir))
        .file(format!("{}/bridge/AccessBridgeDebug.cpp", openjdk_dir));

    let libstdcpp_dir = PathBuf::from(
        String::from_utf8(
            Command::new(build.get_compiler().path())
                .arg("-print-file-name=libstdc++.a")
                .output()
                .expect("failed to query compiler")
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string(),
    )
    .parent()
    .unwrap()
    .to_path_buf();

    build.compile("accessbridge"); // produces libaccessbridge.a

    println!("cargo:rustc-link-search=native={}", libstdcpp_dir.display());
    println!("cargo:rustc-link-lib=static=accessbridge");
    println!("cargo:rustc-link-lib=static=stdc++");

    // 2. Tell cargo to rerun if build.rs changed
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=JAB_JAVA_VERSION");


    // 3. Generate bindings
    let builder = bindgen::Builder::default()
        .header("openjdk/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg(format!("-I{}/bridge", openjdk_dir))
        .clang_arg(format!("-I{}/jni", openjdk_dir))
        .clang_arg(format!("-I{}/jni/linux", openjdk_dir))
        .clang_arg("-Wno-everything")
        .blocklist_type("_LONGDOUBLE")
        .allowlist_file(format!("{}/bridge/AccessBridgePackages.h", openjdk_dir))
        .allowlist_file(format!("{}/bridge/AccessBridgeCallbacks.h", openjdk_dir))
        .allowlist_file(format!("{}/bridge/AccessBridgeCalls.h", openjdk_dir));

    let bindings = builder.generate().expect("Unable to generate bindings");

    // 4. Write bindings to OUT_DIR
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn selected_openjdk_dir() -> String {
    match env::var("JAB_JAVA_VERSION") {
        Ok(n) if n == "latest" => "openjdk/jdk".to_string(),
        Ok(n) => format!("openjdk/jdk{}u", n),
        Err(env::VarError::NotPresent) => "openjdk/jdk8u".to_string(),
        _ => panic!("Failed to get JAB_JAVA_VERSION environment variable"),
    }
}
