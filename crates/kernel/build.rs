fn main() {
    println!("cargo:rerun-if-changed=crates/kernel/x86_64.ld");
    println!("cargo:rustc-link-arg=-Tcrates/kernel/x86_64.ld");
}
