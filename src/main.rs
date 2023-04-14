// use std::io::Read;
use std::collections::HashMap;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long)]
    step: bool,
}

type DataCell = u8;

impl Default for Cli {
    fn default() -> Self {
        Self {
            debug: false,
            step: false,
        }
    }
}

#[derive(Debug)]
enum Command {
    MovRight,
    MovLeft,
    Inc,
    Dec,
    Print,
    Read,
    JumpForward,
    JumpBackward,
    Invalid,
}

impl From<char> for Command {
    fn from(c: char) -> Self {
        match c {
            '>' => Command::MovRight,
            '<' => Command::MovLeft,
            '+' => Command::Inc,
            '-' => Command::Dec,
            '.' => Command::Print,
            ',' => Command::Read,
            '[' => Command::JumpForward,
            ']' => Command::JumpBackward,
            _ => Command::Invalid,
        }
    }
}

struct Brain {
    data: Vec<DataCell>,
    data_pointer: usize,
    program: Vec<char>,
    instruction_pointer: usize,
    output_string: String,
    step: usize,
    bracket_markers: Vec<usize>,
}

impl Brain {
    fn new(program: impl ToString) -> Brain {
        Brain {
            data: vec![0; 30000],
            data_pointer: 0,
            program: program.to_string().chars().collect(),
            instruction_pointer: 0,
            output_string: String::new(),
            step: 0,
            bracket_markers: Vec::new(),
        }
    }

    fn print_debug(&self) {
        if self.instruction_pointer >= self.program.len() {
            return
        }
        println!(
            "Command: {:?} Step: {}",
            Command::from(self.program[self.instruction_pointer]),
            self.step,
        );
        println!("Current byte: {:?}", self.data[self.data_pointer]);

        let data_output = self.data[0..30]
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if i == self.data_pointer {
                    format!("{}*", e)
                } else {
                    e.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        println!("Data: ({}):\n{}", self.data_pointer, data_output);

        let program_string = self
            .program
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("");
        println!("{}", program_string);
        for _ in 0..self.instruction_pointer {
            print!(" ");
        }
        println!("^");
        println!("{}", self.output_string);
    }

    /// Do the next thing
    fn next(&mut self) {
        self.step += 1;

        match Command::from(self.program[self.instruction_pointer]) {
            Command::MovRight => self.mov_right(),
            Command::MovLeft => self.mov_left(),
            Command::Inc => self.inc(),
            Command::Dec => self.dec(),
            Command::Print => self.print(),
            Command::Read => todo!(), //self.read(),
            Command::JumpForward => self.jump_forward(),
            Command::JumpBackward => self.jump_backward(),
            Command::Invalid => {}
        }
        self.instruction_pointer += 1;
    }
    /// Increment the data pointer (to point to the next cell to the right).
    fn mov_right(&mut self) {
        self.data_pointer += 1;
    }
    /// Decrement the data pointer (to point to the next cell to the left).
    fn mov_left(&mut self) {
        if self.data_pointer == 0 {
            return;
        }
        self.data_pointer -= 1;
    }

    /// Increment (increase by one) the byte at the data pointer.
    fn inc(&mut self) {
        if self.data[self.data_pointer] == DataCell::MAX {
            self.data[self.data_pointer] = 0;
        } else {
            self.data[self.data_pointer] += 1;
        }
    }
    /// Decrement (decrease by one) the byte at the data pointer.
    fn dec(&mut self) {
        if self.data[self.data_pointer] == 0 {
            self.data[self.data_pointer] = DataCell::MAX;
        } else {
            self.data[self.data_pointer] -= 1;
        }
    }

    /// Output the byte at the data pointer.
    fn print(&mut self) {
        print!("{}", char::from(self.data[self.data_pointer] as u8));
        self.output_string
            .push(char::from(self.data[self.data_pointer] as u8));
    }

    /// If the byte at the data pointer is zero,
    /// then instead of moving the instruction pointer forward to the next command,
    /// jump it forward to the command after the matching ] command.
    fn jump_forward(&mut self) {
        if self.data[self.data_pointer] == 0 {
            while self.program[self.instruction_pointer] != ']' {
                self.instruction_pointer += 1;
                // print!("{} ", self.instruction_pointer);
            }
            // println!();
        } else {
            // we're in a loop now
            self.bracket_markers.push(self.instruction_pointer);
        }
    }

    /// If the byte at the data pointer is nonzero,
    /// then instead of moving the instruction pointer forward to the next command,
    /// jump it back to the command after the matching [ command.
    fn jump_backward(&mut self) {
        if self.data[self.data_pointer] != 0 {
            // while self.program[self.instruction_pointer] != '[' {
            //     self.instruction_pointer -= 1;
            //     print!("{} ", self.instruction_pointer);
            // }
            if let Some(marker) = self.bracket_markers.last() {
                self.instruction_pointer = marker.to_owned();
            }
        } else {
            self.bracket_markers.pop();
        }
    }
}

fn main() {
    let mut programs = HashMap::new();

    let cli = Cli::parse();


    // this one doesn't :(
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


    programs.insert("test", r#"Calculate the value 256 and test if it's zero
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
    [[-]<]"#);

    let mut brain = Brain::new(programs.get("hello_world").unwrap());
    let stdin = std::io::stdin();

    while brain.instruction_pointer < brain.program.len() {
        brain.next();
        if cli.debug {
            brain.print_debug();
        }
        if cli.step {
            stdin.read_line(&mut String::new()).unwrap();
        }
    }
}
