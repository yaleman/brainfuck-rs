use brainfuck_rs::DataCell;

use crate::Brain;

#[test]
fn hello_world() {
    let logfile = tempfile::NamedTempFile::new().expect("failed to create temp file for log");
    let mut brain = Brain::new(&"++++++++++[>+>+++>+++++++>++++++++++<<<<-]>>>++.>+.+++++++..+++.<<++.>+++++++++++++++.>.+++.------.--------.").with_log_file(logfile.path().into());

    brain.run().expect("program should run without error");

    assert_eq!(brain.output_string().as_ref(), "Hello World");
    assert!(logfile.path().exists(), "log file should exist");
    let log_contents = std::fs::read_to_string(logfile.path()).expect(
        "failed
    to read log file",
    );
    assert!(!log_contents.is_empty(), "log file should not be empty");
}

#[test]
fn nested_loops() {
    let mut brain = Brain::new(&"+++++[>++++++[>++<-]<-]>>+++++.").with_debug(true);

    brain.run().expect("program should run without error");

    assert_eq!(brain.output_string().as_ref(), "A");
}

#[test]
fn dec_from_zero_pointer() {
    let mut brain = Brain::new(&"-.");

    brain.run().expect("program should run without error");

    assert_eq!(brain.data[brain.data_pointer], DataCell::MAX);
}

#[test]
fn inc_from_max_pointer() {
    let mut brain = Brain::new(&"+.");
    // force it to the edge case of incrementing from max to one
    brain.data[brain.data_pointer] = DataCell::MAX;
    brain.run().expect("program should run without error");

    assert_eq!(brain.data[brain.data_pointer], DataCell::MIN);
}

#[test]
fn mov_left_from_zero() {
    let mut brain = Brain::new(&"<.");

    brain.run().expect("program should run without error");

    assert_eq!(brain.data_pointer, 0);
}

#[test]
fn ignore_invalid() {
    let mut brain = Brain::new(&"Hello World.");

    brain.run().expect("program should run without error");

    assert_eq!(brain.data_pointer, 0);
}
