//! Generate and compile the diagnostic C++ bridge from its Rust declaration.
fn main() {
    cxx_build::bridge("src/lib.rs")
        .std("c++17")
        .compile("xdzip-probe-cxx");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/transport.rs");
}
