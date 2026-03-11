use std::borrow::Cow;

pub type DataCell = u8;

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

pub struct Brain {
    pub data: Vec<DataCell>,
    pub data_pointer: usize,
    pub program: Vec<char>,
    pub instruction_pointer: usize,
    pub output_string: Vec<u8>,
    pub step: usize,
    pub debug: bool,
    pub step_mode: bool,
}

impl Brain {
    pub fn new(program: impl ToString) -> Brain {
        Brain {
            data: vec![0; 30000],
            data_pointer: 0,
            program: program.to_string().chars().collect(),
            instruction_pointer: 0,
            output_string: Vec::new(),
            step: 0,
            debug: false,
            step_mode: false,
        }
    }

    pub fn with_debug(self, debug: bool) -> Self {
        Self { debug, ..self }
    }
    pub fn with_step_mode(self, step_mode: bool) -> Self {
        Self { step_mode, ..self }
    }

    pub fn run(&mut self) {
        let stdin = std::io::stdin();

        while self.instruction_pointer < self.program.len() {
            self.next();
            if self.debug {
                self.print_debug();
            }
            if self.step_mode {
                stdin.read_line(&mut String::new()).unwrap();
            }
        }
    }

    pub fn output_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.output_string)
    }

    pub fn print_debug(&self) {
        if self.instruction_pointer >= self.program.len() {
            return;
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
        println!("{}", String::from_utf8_lossy(&self.output_string));
    }

    /// Do the next thing
    pub fn next(&mut self) {
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
        print!("{}", char::from(self.data[self.data_pointer]));
        self.output_string.push(self.data[self.data_pointer]);
    }

    /// If the byte at the data pointer is zero,
    /// then instead of moving the instruction pointer forward to the next command,
    /// jump it forward to the command after the matching ] command.
    fn jump_forward(&mut self) {
        if self.data[self.data_pointer] == 0 {
            let mut depth = 1;

            while depth > 0 {
                self.instruction_pointer += 1;
                if self.program[self.instruction_pointer] == '[' {
                    depth += 1;
                } else if self.program[self.instruction_pointer] == ']' {
                    depth -= 1;
                }
            }
        }
    }

    /// If the byte at the data pointer is nonzero,
    /// then instead of moving the instruction pointer forward to the next command,
    /// jump it back to the command after the matching [ command.
    fn jump_backward(&mut self) {
        if self.data[self.data_pointer] != 0 {
            let mut depth = 1;

            while depth > 0 {
                self.instruction_pointer -= 1;
                if self.program[self.instruction_pointer] == ']' {
                    depth += 1;
                } else if self.program[self.instruction_pointer] == '[' {
                    depth -= 1;
                }
            }
        }
    }
}
