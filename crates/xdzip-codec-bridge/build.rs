//! Generate bounded read/write worker callbacks, without format C++ code.
fn main() {
    cxx_build::bridge("src/lib.rs")
        .include("include")
        .std("c++17")
        .compile("xdzip-codec-cxx");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/host.hpp");
}
