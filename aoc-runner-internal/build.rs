fn main() {
    println!(
        "cargo:rustc-env=PROC_OUT_DIR={}",
        std::env::var("OUT_DIR").unwrap()
    );
}
