fn main() {
    println!("cargo:rerun-if-changed=crates/nd_init/linker.ld");
    println!("cargo:rustc-link-arg=-Tcrates/nd_init/linker.ld");
}
