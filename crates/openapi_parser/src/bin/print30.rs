use openapi_parser::openapi30::parser;

fn main() {
    // Get the file path from the command line arguments
    let path = std::env::args().nth(1).unwrap();

    // Read the file contents into a string
    let openapi = std::fs::read_to_string(&path).unwrap();

    if path.ends_with(".json") {
        match parser::parse_json(&openapi) {
            Ok(openapi) => println!("{:#?}", openapi),
            Err(e) => println!("Unable to parse openapi file:\n\n{:?}", e),
        }
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
        match parser::parse_yaml(&openapi) {
            Ok(openapi) => println!("{:#?}", openapi),
            Err(e) => println!("Unable to parse openapi file:\n\n{:?}", e),
        }
    }
}
