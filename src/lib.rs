#![deny(warnings)]
#![deny(deprecated)]
#![warn(unused_extern_crates)]
// Enable some groups of clippy lints.
#![deny(clippy::suspicious)]
#![deny(clippy::perf)]
// Specific lints to enforce.
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::trivially_copy_pass_by_ref)]
#![deny(clippy::disallowed_types)]
#![deny(clippy::manual_let_else)]
#![allow(clippy::indexing_slicing)] // because otherwise this'd be really slow.

use std::{borrow::Cow, io::Write, path::PathBuf};

pub type DataCell = u8;

const INITIAL_DATA_SIZE: usize = 30_000;

fn is_valid_command(command: u8) -> bool {
    matches!(
        command,
        b'>' | b'<' | b'+' | b'-' | b'.' | b',' | b'[' | b']'
    )
}

fn command_name(command: u8) -> &'static str {
    match command {
        b'>' => "MovRight",
        b'<' => "MovLeft",
        b'+' => "Inc",
        b'-' => "Dec",
        b'.' => "Print",
        b',' => "Read",
        b'[' => "JumpForward",
        b']' => "JumpBackward",
        _ => "Invalid",
    }
}

fn compile_program(program: &str) -> (Vec<u8>, Vec<Option<usize>>, String) {
    let instructions = program
        .bytes()
        .filter(|command| is_valid_command(*command))
        .collect::<Vec<u8>>();
    let mut jump_targets = vec![None; instructions.len()];
    let mut loop_stack = Vec::new();

    for (index, command) in instructions.iter().copied().enumerate() {
        match command {
            b'[' => loop_stack.push(index),
            b']' => {
                if let Some(open_index) = loop_stack.pop() {
                    jump_targets[open_index] = Some(index);
                    jump_targets[index] = Some(open_index);
                }
            }
            _ => {}
        }
    }

    let program_text = instructions
        .iter()
        .map(|command| char::from(*command))
        .collect();

    (instructions, jump_targets, program_text)
}

pub struct Brain {
    pub data: Vec<DataCell>,
    pub data_pointer: usize,
    pub program: Vec<u8>,
    pub instruction_pointer: usize,
    pub output_string: Vec<u8>,
    pub step: usize,
    pub debug: bool,
    pub debug_log: Option<PathBuf>,
    debug_log_handle: Option<std::fs::File>,
    jump_targets: Vec<Option<usize>>,
    program_text: String,
    pub step_mode: bool,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum BrainFucked {
    TooManyLoops,
    InputError(String),
    Io(String),
    Unimplemented(String),
}

impl Brain {
    pub fn new(program: &impl ToString) -> Brain {
        let source = program.to_string();
        let (program, jump_targets, program_text) = compile_program(&source);

        Brain {
            data: vec![0; INITIAL_DATA_SIZE],
            data_pointer: 0,
            program,
            instruction_pointer: 0,
            output_string: Vec::new(),
            step: 0,
            debug: false,
            debug_log: None,
            debug_log_handle: None,
            jump_targets,
            program_text,
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
        let stdout = std::io::stdout();
        let mut stdout = std::io::BufWriter::new(stdout.lock());

        self.run_with_output(&mut stdout)
    }

    // This is here so we can benchmark it while sending everything to a null sink
    pub fn run_with_output(&mut self, output: &mut impl Write) -> Result<(), BrainFucked> {
        let stdin = std::io::stdin();

        if let Some(debug_log) = &self.debug_log {
            match std::fs::File::create(debug_log) {
                Ok(file) => self.debug_log_handle = Some(file),
                Err(err) => {
                    return Err(BrainFucked::Io(format!(
                        "Failed to create debug log file: {err:?}"
                    )))
                }
            }
        }

        let program_len = self.program.len();

        while self.instruction_pointer < program_len {
            self.do_step(output)?;
            if self.debug {
                output.flush().map_err(|err| {
                    BrainFucked::Io(format!("Failed to flush program output: {err:?}"))
                })?;
                self.print_debug();
            }
            if self.step_mode {
                let _ = stdin.read_line(&mut String::new());
            }
        }
        output
            .flush()
            .map_err(|err| BrainFucked::Io(format!("Failed to flush program output: {err:?}")))?;

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
            "Command: {} Step: {}",
            command_name(self.program[self.instruction_pointer]),
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

        buffer.push(self.program_text.clone());
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
    pub fn do_step(&mut self, output: &mut impl Write) -> Result<(), BrainFucked> {
        self.step += 1;

        match self.program[self.instruction_pointer] {
            b'>' => self.mov_right(),
            b'<' => self.mov_left(),
            b'+' => self.inc(),
            b'-' => self.dec(),
            b'.' => self.print(output)?,
            b',' => self.read()?,
            b'[' => self.jump_forward()?,
            b']' => self.jump_backward()?,
            _ => {}
        }
        self.instruction_pointer += 1;
        Ok(())
    }

    /// Increment the data pointer (to point to the next cell to the right).
    #[inline(always)]
    fn mov_right(&mut self) {
        self.data_pointer = self.data_pointer.saturating_add(1);
        if self.data_pointer >= self.data.len() {
            self.data.resize(self.data.len() * 2, 0);
        }
    }
    /// Decrement the data pointer (to point to the next cell to the left).
    #[inline(always)]
    fn mov_left(&mut self) {
        self.data_pointer = self.data_pointer.saturating_sub(1);
    }

    fn read(&mut self) -> Result<(), BrainFucked> {
        Err(BrainFucked::Unimplemented(
            "Read isn't implemented yet!".to_string(),
        ))
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
    fn print(&mut self, output: &mut impl Write) -> Result<(), BrainFucked> {
        let byte = self.data[self.data_pointer];
        output
            .write_all(&[byte])
            .map_err(|err| BrainFucked::Io(format!("Failed to write program output: {err:?}")))?;
        self.output_string.push(byte);

        Ok(())
    }

    /// If the byte at the data pointer is zero,
    /// then instead of moving the instruction pointer forward to the next command,
    /// jump it forward to the command after the matching ] command.
    fn jump_forward(&mut self) -> Result<(), BrainFucked> {
        if self.data[self.data_pointer] == 0 {
            let Some(target) = self.jump_targets[self.instruction_pointer] else {
                return Err(BrainFucked::InputError(
                    "Unmatched '[' instruction".to_string(),
                ));
            };
            self.instruction_pointer = target;
        }
        Ok(())
    }

    /// If the byte at the data pointer is nonzero,
    /// then instead of moving the instruction pointer forward to the next command,
    /// jump it back to the command after the matching [ command.
    fn jump_backward(&mut self) -> Result<(), BrainFucked> {
        if self.data[self.data_pointer] != 0 {
            let Some(target) = self.jump_targets[self.instruction_pointer] else {
                return Err(BrainFucked::InputError(
                    "Unmatched ']' instruction".to_string(),
                ));
            };
            self.instruction_pointer = target;
        }

        Ok(())
    }
}
