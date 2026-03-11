// use std::io::Read;
use std::collections::HashMap;

use brainfuck_rs::Brain;
use clap::Parser;

#[cfg(test)]
mod test;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long)]
    step: bool,
    #[arg(short, long, help = "program to run")]
    program: Option<String>,
    #[arg(long, help = "list built-in programs")]
    list: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            debug: false,
            step: false,
            program: Some("add two and five".to_string()),
            list: false,
        }
    }
}

fn main() {
    let mut programs = HashMap::new();

    let cli = Cli::parse();

    // this one doesn't  :(
    programs.insert("hello_world", "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");

    // this one works
    programs.insert(
        "add_two_and_five",
        "++       Cell c0 = 2
    > +++++  Cell c1 = 5
    [        Start your loops with your cell pointer on the loop counter (c1 in our case)
    < +      Add 1 to c0
    > -      Subtract 1 from c1
    ]    ",
    );

    programs.insert(
        "test",
        r#"Calculate the value 256 and test if it's zero
    If the interpreter errors on overflow this is where it'll happen
    ++++++++[>++++++++<-]>[<++++>-]
    +<[>-<
        Not zero so multiply by 256 again to get 65536
        [>++++<-]>[<++++++++>-]<[>++++++++<-]
        +>[>
            # Print "32"
            ++++++++++[>+++++<-]>+.-.[-]<
        <[-]<->] <[>>
            # Print "16"
            +++++++[>+++++++<-]>.+++++.[-]<
    <<-]] >[>
        # Print "8"
        ++++++++[>+++++++<-]>.[-]<
    <-]<
    # Print " bit cells\n"
    +++++++++++[>+++>+++++++++>+++++++++>+<<<<-]>-.>-.+++++++.+++++++++++.<.
    >>.++.+++++++..<-.>>-
    Clean up used cells.
    [[-]<]"#,
    );

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
            brain.run();
        } else {
            println!("Program not found: {}", program_name);
        }
    }
}
