use std::sync::Arc;

fn main() {
    Box::new(10i32);
    Box::new(20i64);
    Box::new(30i128);
    vec![1u8, 2, 3];

    Arc::new(2u8);
    String::from("hello");
    String::from("h");
}
