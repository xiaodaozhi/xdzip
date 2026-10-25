//! Generate the worker's bounded read-only stream bridge, without format C++ code.
fn main() {
    cxx_build::bridge("src/lib.rs")
        .include("include")
        .std("c++17")
        .compile("xdzip-readonly-cxx");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/host.hpp");
}
