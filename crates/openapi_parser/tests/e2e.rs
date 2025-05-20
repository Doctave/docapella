mod test {
    static PETSTORE_SPEC: &str = include_str!("./fixtures/petstore.json");
    static GITHUB_SPEC: &str =
        include_str!("./../../libdoctave/examples/open_api_specs/github.yaml");

    #[test]
    fn parses_petstore() {
        let _spec = openapi_parser::openapi30::parser::parse_json(PETSTORE_SPEC).unwrap();
    }

    #[test]
    fn parses_github() {
        let _spec = openapi_parser::openapi30::parser::parse_yaml(GITHUB_SPEC).unwrap();
    }
}
