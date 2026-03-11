use brainfuck_rs::DataCell;

use crate::Brain;

#[test]
fn test_hello_world() {
    let mut brain = Brain::new("++++++++++[>+>+++>+++++++>++++++++++<<<<-]>>>++.>+.+++++++..+++.<<++.>+++++++++++++++.>.+++.------.--------.");

    brain.run().expect("program should run without error");

    assert_eq!(brain.output_string().as_ref(), "Hello World");
}

#[test]
fn test_nested_loops() {
    let mut brain = Brain::new("+++++[>++++++[>++<-]<-]>>+++++.").with_debug(true);

    brain.run().expect("program should run without error");

    assert_eq!(brain.output_string().as_ref(), "A");
}

#[test]
fn test_dec_from_zero_pointer() {
    let mut brain = Brain::new("-.");

    brain.run().expect("program should run without error");

    assert_eq!(brain.data[brain.data_pointer], DataCell::MAX);
}

#[test]
fn test_inc_from_max_pointer() {
    let mut brain = Brain::new("+.");
    // force it to the edge case of incrementing from max to one
    brain.data[brain.data_pointer] = DataCell::MAX;
    brain.run().expect("program should run without error");

    assert_eq!(brain.data[brain.data_pointer], DataCell::MIN);
}

#[test]
fn test_mov_left_from_zero() {
    let mut brain = Brain::new("<.");

    brain.run().expect("program should run without error");

    assert_eq!(brain.data_pointer, 0);
}
