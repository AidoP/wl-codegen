use heck::ToKebabCase;
use std::{io::Write, fs::File, process::Command};

const PROTO_DIR: &'static str = "src/wayland/proto";
const PROTOCOLS: &'static [&'static str] = &[
    "wayland",
    "xdg_shell",
    "linux_dmabuf_unstable_v1"
];

fn main() {
    // Generate the proto module to import the generated code
    let mod_path = &format!("{PROTO_DIR}/mod.rs");
    let mut proto_mod = match File::create(mod_path) {
        Ok(proto_mod) => proto_mod,
        Err(error) => panic!("Failed to create Rust source file '{mod_path}': {error:?}")
    };
    if let Err(error) = writeln!(proto_mod, "// Auto-Generated file. Do not edit.") {
        panic!("Failed to write Rust source file '{mod_path}': {error:?}")
    }
    // Generate Wayland dispatch glue
    for protocol in PROTOCOLS {
        if let Err(error) = writeln!(proto_mod, "mod {protocol};\npub use {protocol}::*;") {
            panic!("Failed to write Rust source file '{mod_path}': {error:?}")
        }
        wl_codegen(protocol)
    }
}

fn wl_codegen(protocol: &str) {
    let spec = &format!("protocol/{}.toml", protocol.to_kebab_case());
    let proto = &format!("{PROTO_DIR}/{protocol}.rs");

    println!("cargo:rerun-if-changed={spec}");
    println!("cargo:rerun-if-changed={proto}");

    let code = match yutani_codegen::protocol(spec) {
        Ok(code) => code,
        Err(error) => panic!("Failed to read protocol specification '{spec}': {error:?}")
    };
    let mut proto_file = match File::create(proto) {
        Ok(proto_file) => proto_file,
        Err(error) => panic!("Failed to create Rust source file '{proto}': {error:?}")
    };
    if let Err(error) = writeln!(proto_file, "// Auto-Generated file. Do not edit.\n#![allow(dead_code)]\n\n{}", code) {
        panic!("Failed to write Rust source file '{proto}': {error:?}")
    }
    if let Err(error) = Command::new("rustfmt").arg(proto).status() {
        panic!("Failed to run rustfmt on Rust source file '{proto}': {error:?}")
    }
}