use std::path::{Path, PathBuf};

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

const PROGRAMS_DIR: &str = "./programs";
const PROGRAM_EXTENSION: &str = "b";

fn is_program_file(path: &Path) -> bool {
    path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some(PROGRAM_EXTENSION)
}

fn list_programs() -> std::io::Result<Vec<String>> {
    let mut programs = Vec::new();

    for entry in std::fs::read_dir(PROGRAMS_DIR)? {
        let path = entry?.path();
        if is_program_file(&path) {
            if let Some(name) = path.file_stem().and_then(std::ffi::OsStr::to_str) {
                programs.push(name.to_string());
            }
        }
    }

    programs.sort_unstable();
    Ok(programs)
}

fn load_program(program_name: &str) -> std::io::Result<String> {
    let path = Path::new(PROGRAMS_DIR).join(format!("{program_name}.{PROGRAM_EXTENSION}"));

    std::fs::read_to_string(path)
}

fn main() {
    let cli = Cli::parse();

    if cli.list {
        println!("Built-in programs:");
        match list_programs() {
            Ok(programs) => {
                for name in programs {
                    println!("- {}", name);
                }
            }
            Err(err) => eprintln!("Failed to list built-in programs: {err:?}"),
        }
    } else {
        let program_name = cli.program.unwrap_or("add_two_and_five".to_string());
        match load_program(&program_name) {
            Ok(program) => {
                eprintln!("Running program: '{}'", program_name);
                let mut brain = Brain::new(&program)
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
            }
            Err(err) => eprintln!(
                "Program not found or unreadable '{}': {err:?}",
                program_name
            ),
        }
    }
}
