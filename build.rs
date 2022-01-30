use std::convert::TryInto;
use std::env;
use std::env::VarError;
use std::error::Error;
use std::fs::{DirEntry, File, read_dir};
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let grammars = vec!["CSV", "ReferenceToATN", "XMLLexer", "SimpleLR", "Labels"];
    let antlr_path = "/home/rrevenantt/dev/antlr4/tool/target/antlr4-4.8-2-SNAPSHOT-complete.jar";

    for grammar in grammars {
        //ignoring error because we do not need to run anything when deploying to crates.io
        let _ = gen_for_grammar(grammar, antlr_path);
    }

    println!("cargo:rerun-if-changed=build.rs");

    println!("cargo:rerun-if-changed=/home/rrevenantt/dev/antlr4/tool/target/antlr4-4.8-2-SNAPSHOT-complete.jar");
}

fn gen_for_grammar(grammar_file_name: &str, antlr_path: &str) -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir);

    let input = env::current_dir().unwrap().join("grammars");
    let file_name = grammar_file_name.to_owned() + ".g4";

    let x = Command::new("java")
        .current_dir(input)
        // .arg("-jar")
        .arg("-cp")
        .arg(antlr_path)
        .arg("org.antlr.v4.Tool")
        .arg("-Dlanguage=Rust")
        .arg("-no-visitor")
        .arg("-o")
        .arg("../tests/gen")
        .arg(&file_name)
        .spawn()
        .expect("antlr tool failed to start")
        .wait_with_output()?;
    // .unwrap()
    // .stdout;
    // eprintln!("xx{}",String::from_utf8(x).unwrap());

    println!("cargo:rerun-if-changed=grammars/{}", file_name);
    Ok(())
}