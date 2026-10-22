//! Generate the read-only service bridge; the diagnostic probe remains independent.
fn main() {
    cxx_build::bridges(["src/lib.rs", "src/update.rs"])
        .std("c++17")
        .compile("xdzip-reader-cxx");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/update.rs");
}
