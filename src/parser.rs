use crate::error::{Error, Result};

#[derive(Debug)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub struct JsonParser<'a> {
    json_string: &'a str,
    index: usize,
}

impl<'a> JsonParser<'a> {
    pub fn new(json_string: &'a str) -> Self {
        JsonParser {
            json_string,
            index: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        self.consume_whitespace();

        self.sanity_check()?;

        let result = self.parse_value()?;
        self.consume_whitespace();

        if self.index != self.json_string.len() {
            return Err(Error::UnexcpectedCharacters());
        }

        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue> {
        self.consume_whitespace();

        let next_char = self.json_string.chars().nth(self.index);

        match next_char {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some(c) if c.is_ascii_digit() || c == '-' => self.parse_number(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null(),
            _ => Err(Error::UnexcpectedCharacters()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.consume('{');
        let mut result = Vec::new();

        while self.json_string.chars().nth(self.index) != Some('}') {
            self.consume_whitespace();

            let key = match self.parse_string()? {
                JsonValue::String(val) => val,
                _ => return Err(Error::UnexcpectedCharacters()),
            };

            // Cases when key is empty
            if key.is_empty() {
                self.consume('"');
            }

            self.consume_whitespace();

            self.expect(':')?;
            self.consume(':');
            self.consume_whitespace();

            let value = self.parse_value()?;

            result.push((key, value));
            self.consume('"');

            self.consume_whitespace();

            if self.json_string.chars().nth(self.index) == Some(',') {
                self.consume(',');
                self.consume('\n');
            } else if self.json_string.chars().nth(self.index) != Some('}') {
                return Err(Error::InvalidInput("Expected '}' or ','".to_string()));
            }

            self.consume_whitespace();
        }

        self.consume('}');

        Ok(JsonValue::Object(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.consume('[');
        let mut result = Vec::new();

        while self.json_string.chars().nth(self.index) != Some(']') {
            let value = self.parse_value()?;
            result.push(value);

            self.consume_whitespace();

            if self.json_string.chars().nth(self.index) == Some(',') {
                self.consume(',');
            } else if self.json_string.chars().nth(self.index) != Some(']') {
                return Err(Error::InvalidInput("Expected ']' or ','".to_string()));
            }
        }

        self.consume(']'); // Consume ']'
        Ok(JsonValue::Array(result))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue> {
        self.consume_whitespace();

        let next_char = self.json_string.chars().nth(self.index);

        if next_char == Some('t') {
            self.index += 4;
            Ok(JsonValue::Boolean(true))
        } else if next_char == Some('f') {
            self.index += 5;
            Ok(JsonValue::Boolean(false))
        } else {
            Err(Error::InvalidInput("Invalid boolean value".to_string()))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue> {
        let start = self.index;
        let mut has_dot = false;

        while let Some(c) = self.json_string.chars().nth(self.index) {
            if c == '.' || c == 'e' {
                has_dot = true;
            } else if !c.is_ascii_digit() && c != 'e' && c != 'E' && c != '-' && c != '+' {
                break;
            }

            self.index += 1;
        }

        let number_str = &self.json_string[start..self.index];

        if has_dot {
            match number_str.parse::<f64>() {
                Ok(num) => Ok(JsonValue::Number(num)),
                Err(_) => Err(Error::InvalidInput("Invalid number".to_string())),
            }
        } else {
            if self.json_string.chars().nth(start) == Some('0') && number_str.len() > 1 {
                return Err(Error::InvalidInput("Cant start with 0".to_string()));
            }

            match number_str.parse::<i64>() {
                Ok(num) => Ok(JsonValue::Number(num as f64)),
                Err(_) => Err(Error::InvalidInput("Invalid number".to_string())),
            }
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue> {
        self.consume('"');
        self.consume_whitespace();
        let start = self.index;

        while let Some(c) = self.json_string.chars().nth(self.index) {
            self.index += 1;

            if c == '\\' {
                let next_charachter = self.json_string.chars().nth(self.index);
                if !Self::is_valid_escape(next_charachter.unwrap()) {
                    return Err(Error::InvalidInput("Invalid escape_char".to_string()));
                }
                self.index += 1;

                continue;
            }

            if c == '\t' {
                return Err(Error::InvalidInput("Tab character in string".to_string()));
            }

            if c == '\n' {
                return Err(Error::InvalidInput("Line break in string".to_string()));
            }

            if c == '"' {
                return Ok(JsonValue::String(
                    self.json_string[start..self.index - 1].to_string(),
                ));
            }
        }

        Err(Error::InvalidInput("Unterminated string".to_string()))
    }

    fn is_valid_escape(c: char) -> bool {
        matches!(c, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u')
    }

    fn parse_null(&mut self) -> Result<JsonValue> {
        let next_char = self.json_string.chars().nth(self.index);

        if next_char == Some('n') {
            self.index += 4;
            Ok(JsonValue::Null)
        } else {
            Err(Error::InvalidInput("Invalid null value".to_string()))
        }
    }

    fn consume(&mut self, ch: char) {
        if Some(ch) == self.json_string.chars().nth(self.index) {
            self.index += 1;
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.json_string.chars().nth(self.index) {
            if !c.is_whitespace() {
                break;
            }
            self.index += 1;
        }
    }

    fn expect(&mut self, expected: char) -> Result<()> {
        if self.json_string.chars().nth(self.index) == Some(expected) {
            Ok(())
        } else {
            Err(Error::InvalidInput(format!(
                "Expected: {expected} and not found"
            )))
        }
    }

    fn sanity_check(&self) -> Result<()> {
        let first_char = match self.json_string.chars().next() {
            Some(c) => c,
            None => return Err(Error::InvalidInput("Invalid input".to_string())),
        };

        match first_char {
            '{' | '[' => {
                // Check for trailing commas
                if self.json_string.ends_with(",}") || self.json_string.ends_with(",]") {
                    return Err(Error::InvalidInput("Trailing comma detected".to_string()));
                }

                Ok(())
            }
            _ => Err(Error::InvalidInput(
                "Json should be an object or array".to_string(),
            )),
        }
    }
}
