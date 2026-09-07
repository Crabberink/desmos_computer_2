use std::{fs::File, io::Write};

use clap::Parser;

use crate::cli::Cli;

mod cli;
mod compiler;

fn main() {
    let cli = Cli::parse();

    let Ok(source)=  std::fs::read_to_string(&cli.file) else {
        println!("Failed to read file: {}", cli.file.display());
        return;
    };

    let output = match compiler::compile_source(&source) {
        Err(errors) => {
            for error in errors {
                println!("{}",error);
            }
            return;
        },
        Ok(output) => output,
    };

    let out_path = &mut cli.file.clone();
    out_path.set_extension("txt");

    let mut out_file = File::create(out_path.clone()).expect("Failed to create output file!");
    out_file.write_all(&output.into_bytes()).expect("Failed to write to output file");

    println!("Compiled {} to {}", &cli.file.to_str().expect("Failed to convert input path to string?"), out_path.to_str().expect("Failed to convert output path to string?"));
}
