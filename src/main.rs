use std::{collections::HashMap, path::PathBuf};

use brainfuck_rs::Brain;
use clap::Parser;

#[cfg(test)]
mod test;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    debug: bool,
    #[arg(
        long,
        help = "Debug log to a file, overwrites on run, sets debug -> true"
    )]
    debug_file: Option<PathBuf>,
    #[arg(short, long)]
    step: bool,
    #[arg(short, long, help = "program to run")]
    program: Option<String>,
    #[arg(long, help = "list built-in programs")]
    list: bool,
}

fn main() {
    let mut programs = HashMap::new();

    let cli = Cli::parse();

    for file in std::fs::read_dir("./programs").expect("failed to read programs directory") {
        let file = file.expect("failed to read program file");
        let path = file.path();
        if path.is_file() && path.display().to_string().ends_with(".b") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                let contents = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("failed to read program file: {:?}", path));
                programs.insert(name.to_string(), contents);
            }
        } else {
            eprintln!(
                "Skipping non-file or non-.b entry in programs directory: {:?}",
                path
            );
        }
    }

    if cli.list {
        println!("Built-in programs:");
        for name in programs.keys() {
            println!("- {}", name);
        }
    } else {
        let program_name = cli.program.unwrap_or("add_two_and_five".to_string());
        if let Some(program) = programs.get(program_name.as_str()) {
            eprintln!("Running program: '{}'", program_name);
            let mut brain = Brain::new(program)
                .with_debug(cli.debug)
                .with_step_mode(cli.step);
            if let Some(debug_file) = cli.debug_file {
                brain = brain.with_log_file(debug_file);
            }
            if let Err(err) = brain.run() {
                eprintln!("Error running program: {:?}", err);
            } else {
                eprintln!("Program ran OK!");
            };
        } else {
            println!("Program not found: {}", program_name);
        }
    }
}
