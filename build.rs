fn main() {
    // This tells cargo to rerun this build script if the WIT file changes
    println!("cargo:rerun-if-changed=spin-mcp.wit");
}