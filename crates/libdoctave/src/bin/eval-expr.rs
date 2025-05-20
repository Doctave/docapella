use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let program = args.get(1).expect("No program supplied");

    let start = Instant::now();

    let ast = match libdoctave::markdown::expressions::parse(program) {
        Ok(a) => a,
        Err(err) => {
            let duration = start.elapsed();
            println!("--------------------------------");
            println!("Error parsing AST ({:?})\n", duration);
            println!("{}", err);
            std::process::exit(1);
        }
    };
    let duration = start.elapsed();

    println!("--------------------------------");
    println!("Parsed AST in {:?}\n", duration);
    println!("{:#?}", ast);

    let start = Instant::now();
    let mut interpreter = libdoctave::markdown::expressions::Interpreter::new(None);
    let out = match interpreter.interpret(ast) {
        Ok(val) => val.to_string(),
        Err(err) => err.to_string(),
    };
    let duration = start.elapsed();

    println!("--------------------------------");
    println!("Evaluated AST in {:?}\n", duration);
    println!("{}", out);
}
