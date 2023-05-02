fn main() {
    println!("cargo:rerun-if-changed=crates/nd_init/x86_64.ld");
    println!("cargo:rustc-link-arg=-Tcrates/nd_init/x86_64.ld");
}
