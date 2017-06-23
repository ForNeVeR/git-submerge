extern crate rustc_version;
use rustc_version::{version, Version};

fn main() {
    if version().unwrap() >= Version::parse("1.19.0").unwrap() {
        println!("cargo:rustc-cfg=HAS_EPRINTLN");
    }
}
