use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("stdin");
        let input: String = serde_json::from_str(&line).expect("JSON string");
        let result = tezbar_asciimath_to_latex_converter::convert(&input);
        println!("{}", serde_json::to_string(&result).expect("result"));
    }
}
