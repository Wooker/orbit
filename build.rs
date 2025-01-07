fn main() {
    println!("cargo::warning=HELLO");
    println!("cargo::rerun-if-changed=build.rs");
}
