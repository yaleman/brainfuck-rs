use crate::Brain;

#[test]
fn test_hello_world() {
    let mut brain = Brain::new("++++++++++[>+>+++>+++++++>++++++++++<<<<-]>>>++.>+.+++++++..+++.<<++.>+++++++++++++++.>.+++.------.--------.");

    brain.run();

    assert_eq!(brain.output_string().as_ref(), "Hello World");
}
