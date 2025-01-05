fn main() {
    let data = utils::parser::Parser::new()
        .parse_file("/tmp/pipe.out")
        .unwrap();
}
