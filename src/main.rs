pub mod tokenize;
pub mod tree;

fn main() {
    let input = std::fs::read_to_string("test1.fnx").expect("failed to read input file");
    let tokens = tokenize::tokenize(input).expect("failed to tokenize");
    let tree = tree::make_tree(tokens).expect("failed to make tree");
    println!("{:?}", tree);
}
