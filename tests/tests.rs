#[macro_export]
macro_rules! assert_parser {
    ($path:expr, success) => {{
        let content = fs::read_to_string($path).unwrap();

        let mut parser = JsonParser::new(&content);

        let result = parser.parse();
        assert!(result.is_ok());
    }};
    ($path:expr, fail) => {{
        let content = fs::read_to_string($path).unwrap();

        let mut parser = JsonParser::new(&content);

        assert!(parser.parse().is_err());
    }};
}

#[cfg(test)]
mod tests {
    use json_parser::JsonParser;
    use std::fs;

    #[test]
    fn simple() {
        let json_str = r#"
        {
            "name": "John",
            "surname": "Doe",
            "age": 15,
            "id": 555555,
            "is_student": true,
            "grades": [9, 8.5, 9.5, 10],
            "address": {
                "city": "Limassol",
                "zipcode": "4141"
            },
            "null": null,
        }
    "#;

        let mut parser = JsonParser::new(json_str);

        match parser.parse() {
            Ok(result) => println!("result: {:#?}\n", result),

            Err(err) => println!("Error: {}", err),
        }
    }

    #[test]
    fn advance() {
        assert_parser!("tests/inputs/pass1.json", success);
        assert_parser!("tests/inputs/pass2.json", success);
        assert_parser!("tests/inputs/pass3.json", success);

        for i in 1..=33 {
            let path = format!("tests/inputs/fail{i}.json");
            if std::path::Path::new(&path).exists() {
                assert_parser!(path, fail);
            }
        }
    }
}
