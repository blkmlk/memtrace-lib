fn main() {
    Box::new(10i32);
    Box::new(20i64);
    Box::new(30i128);

    let _: Vec<u8> = Vec::with_capacity(3);

    String::from("hello");
    String::from("h");
}
