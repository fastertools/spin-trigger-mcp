fn main() {
    // This tells cargo to rerun this build script if the WIT file changes
    println!("cargo:rerun-if-changed=spin-mcp.wit");
    
    // This is needed for musl builds (e.g., Linux static builds)
    #[cfg(target_env = "musl")]
    {
        // Force linking against OpenSSL for musl builds
        if let Ok(lib_dir) = std::env::var("OPENSSL_LIB_DIR") {
            println!("cargo:rustc-link-search={}", lib_dir);
        }
    }
}