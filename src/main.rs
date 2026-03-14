pub mod tokenize;
pub mod tree;

fn main() {
    let input = std::fs::read_to_string("test1.fnx").expect("failed to read input file");
    let tokens = tokenize::tokenize(input).expect("failed to tokenize");
    println!("{:?}", tokens);
}
