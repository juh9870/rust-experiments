use chumsky::Parser;

pub mod ast;
pub mod errors;
pub mod parser;
pub mod value;
pub mod vm;

fn main() {
    let p = parser::lexer();
    let parsed = "\
fib = function(n) //dumb recursive approach
    if n < 2 then return 1
    return fib(n-1) + fib(n-2)
end function

// stress test
fib 28
";
    let result = p.parse(parsed);
//     let result = p.parse("\n");
    println!("\n\n{result:?}\n\n");
    // println!("{}", result.unwrap()[0].0)
}
