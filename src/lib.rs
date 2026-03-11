use std::{borrow::Cow, path::PathBuf};

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
    pub debug_log: Option<PathBuf>,
    debug_log_handle: Option<std::fs::File>,
    pub step_mode: bool,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum BrainFucked {
    TooManyLoops,
    InputError(String),
    Io(String),
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
            debug_log: None,
            debug_log_handle: None,
            step_mode: false,
        }
    }

    pub fn with_debug(self, debug: bool) -> Self {
        Self { debug, ..self }
    }
    pub fn with_step_mode(self, step_mode: bool) -> Self {
        Self { step_mode, ..self }
    }

    pub fn with_log_file(self, path: PathBuf) -> Self {
        Self {
            debug_log: Some(path),
            debug: true,
            ..self
        }
    }

    pub fn run(&mut self) -> Result<(), BrainFucked> {
        let stdin = std::io::stdin();

        if let Some(debug_log) = &self.debug_log {
            match std::fs::File::create(debug_log) {
                Ok(file) => self.debug_log_handle = Some(file),
                Err(err) => eprintln!("Failed to create debug log file: {err:?}"),
            }
        }

        let program_len = self.program.len();

        while self.instruction_pointer < program_len {
            self.do_step()?;
            if self.debug {
                self.print_debug();
            }
            if self.step_mode {
                let _ = stdin.read_line(&mut String::new());
            }
        }
        Ok(())
    }

    pub fn output_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.output_string)
    }

    pub fn print_debug(&mut self) {
        if self.instruction_pointer >= self.program.len() {
            return;
        }
        let mut buffer = Vec::new();

        buffer.push(format!(
            "Command: {:?} Step: {}",
            Command::from(self.program[self.instruction_pointer]),
            self.step,
        ));
        buffer.push(format!("Current byte: {:?}", self.data[self.data_pointer]));

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
        buffer.push(format!("Data: ({}):\n{}", self.data_pointer, data_output));

        let program_string = self
            .program
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("");
        buffer.push(program_string.to_string());
        let mut pointer_buf = String::new();
        for _ in 0..self.instruction_pointer {
            pointer_buf.push(' ');
        }
        buffer.push(format!("{}^", pointer_buf));
        buffer.push(format!("{}", String::from_utf8_lossy(&self.output_string)));

        if let Some(debug_log_handle) = &mut self.debug_log_handle {
            use std::io::Write;
            if let Err(err) = writeln!(debug_log_handle, "{}", buffer.join("\n")) {
                eprintln!("Failed to write to debug log: {err:?}");
            }
        } else {
            println!("{}", buffer.join("\n"));
        }
    }

    /// Do the next thing
    pub fn do_step(&mut self) -> Result<(), BrainFucked> {
        self.step += 1;

        match Command::from(self.program[self.instruction_pointer]) {
            Command::MovRight => self.mov_right(),
            Command::MovLeft => self.mov_left(),
            Command::Inc => self.inc(),
            Command::Dec => self.dec(),
            Command::Print => self.print(),
            Command::Read => self.read()?,
            Command::JumpForward => self.jump_forward()?,
            Command::JumpBackward => self.jump_backward(),
            Command::Invalid => {}
        }
        self.instruction_pointer += 1;
        Ok(())
    }

    /// Increment the data pointer (to point to the next cell to the right).
    #[inline(always)]
    fn mov_right(&mut self) {
        self.data_pointer = self.data_pointer.saturating_add(1);
        if self.data_pointer >= self.data.capacity() {
            eprintln!("Resizing memory to {}", self.data.len() * 2);
            self.data.resize(self.data.len() * 2, 0);
        }
    }
    /// Decrement the data pointer (to point to the next cell to the left).
    #[inline(always)]
    fn mov_left(&mut self) {
        self.data_pointer = self.data_pointer.saturating_sub(1);
    }

    fn read(&mut self) -> Result<(), BrainFucked> {
        let mut buffer = [0u8; 1];
        use std::io::Read;
        match std::io::stdin().read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    Ok(())
                } else {
                    self.data[self.data_pointer] = buffer[0];
                    Ok(())
                }
            }
            Err(err) => Err(BrainFucked::InputError(format!(
                "Failed to read input: {err:?}"
            ))),
        }
    }

    /// Increment (increase by one) the byte at the data pointer.
    #[inline(always)]
    fn inc(&mut self) {
        self.data[self.data_pointer] = self.data[self.data_pointer].wrapping_add(1);
    }
    /// Decrement (decrease by one) the byte at the data pointer.
    #[inline(always)]
    fn dec(&mut self) {
        self.data[self.data_pointer] = self.data[self.data_pointer].wrapping_sub(1);
    }

    /// Output the byte at the data pointer.
    #[inline(always)]
    fn print(&mut self) {
        print!("{}", char::from(self.data[self.data_pointer]));
        self.output_string.push(self.data[self.data_pointer]);
    }

    /// If the byte at the data pointer is zero,
    /// then instead of moving the instruction pointer forward to the next command,
    /// jump it forward to the command after the matching ] command.
    fn jump_forward(&mut self) -> Result<(), BrainFucked> {
        if self.data[self.data_pointer] == 0 {
            let mut depth = 1;

            let maxloop = 1000000;
            let mut current_loop = 0;
            while depth > 0 {
                self.instruction_pointer = self.instruction_pointer.saturating_add(1);
                if self.program[self.instruction_pointer] == '[' {
                    depth += 1;
                } else if self.program[self.instruction_pointer] == ']' {
                    depth -= 1;
                }
                current_loop += 1;
                if current_loop > maxloop {
                    return Err(BrainFucked::TooManyLoops);
                }
            }
        }
        Ok(())
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
