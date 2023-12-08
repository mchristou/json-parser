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

    pub fn parse(&mut self) -> Result<JsonValue, &'static str> {
        self.consume_whitespace();

        let result = self.parse_value()?;
        self.consume_whitespace();

        if self.index != self.json_string.len() {
            return Err("Unexpected characters");
        }

        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, &'static str> {
        self.consume_whitespace();

        let next_char = self.json_string.chars().nth(self.index);

        match next_char {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some(c) if c.is_ascii_digit() || c == '-' => self.parse_number(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null(),
            _ => Err("Unexpected charachter"),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, &'static str> {
        self.index += 1; // Consume '{'
        let mut result = Vec::new();

        while self.json_string.chars().nth(self.index) != Some('}') {
            self.consume_whitespace();

            let key = match self.parse_string()? {
                JsonValue::String(val) => val,
                _ => {
                    return Err("Unexpected charachter");
                }
            };

            // Cases when key is empty
            if key.is_empty() {
                self.consume('"');
            }

            self.consume_whitespace();

            if self.json_string.chars().nth(self.index) != Some(':') {
                return Err("Expected ':'");
            }
            self.index += 1; // Consume ':'
            self.consume_whitespace();

            let value = self.parse_value()?;

            result.push((key, value));
            self.consume('"');

            self.consume_whitespace();

            if self.json_string.chars().nth(self.index) == Some(',') {
                self.consume(',');
                self.consume('\n');
            } else if self.json_string.chars().nth(self.index) != Some('}') {
                return Err("Expected '}' or ','");
            }

            self.consume_whitespace();
        }

        self.index += 1; // Consume '}'
        Ok(JsonValue::Object(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, &'static str> {
        self.index += 1; // Consume '['
        let mut result = Vec::new();

        while self.json_string.chars().nth(self.index) != Some(']') {
            let value = self.parse_value()?;
            result.push(value);

            self.consume_whitespace();

            if self.json_string.chars().nth(self.index) == Some(',') {
                self.index += 1; // Consume ','
            } else if self.json_string.chars().nth(self.index) != Some(']') {
                return Err("Expected ']' or ','");
            }
        }

        self.index += 1; // Consume ']'
        Ok(JsonValue::Array(result))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, &'static str> {
        self.consume_whitespace();

        let next_char = self.json_string.chars().nth(self.index);

        if next_char == Some('t') {
            self.index += 4; // Consume 'true'
            Ok(JsonValue::Boolean(true))
        } else if next_char == Some('f') {
            self.index += 5; // Consume 'false'
            Ok(JsonValue::Boolean(false))
        } else {
            Err("Invalid boolean value")
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, &'static str> {
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
        println!("number_str: {number_str}");

        if has_dot {
            match number_str.parse::<f64>() {
                Ok(num) => Ok(JsonValue::Number(num)),
                Err(_) => Err("Invalid number"),
            }
        } else {
            match number_str.parse::<i64>() {
                Ok(num) => Ok(JsonValue::Number(num as f64)),
                Err(_) => Err("Invalid number"),
            }
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, &'static str> {
        self.index += 1; // Consume '"'
        self.consume_whitespace();
        let start = self.index;
        let mut escape_char = vec![];

        while let Some(c) = self.json_string.chars().nth(self.index) {
            self.index += 1;

            if c == '\\' {
                escape_char.push(self.index);
                self.index += 1;
                continue;
            }

            if c == '"' && !escape_char.contains(&(self.index - 1)) {
                if !escape_char.is_empty() {
                    let result =
                        self.filter_escape_charachters((start, self.index - 1), escape_char);
                    return Ok(JsonValue::String(result));
                }
                return Ok(JsonValue::String(
                    self.json_string[start..self.index - 1].to_string(),
                ));
            }
        }

        Err("Unterminated string")
    }

    fn filter_escape_charachters(
        &self,
        input_range: (usize, usize),
        indices: Vec<usize>,
    ) -> String {
        // Filter out indices to remove from the substring range
        let range = input_range.0..input_range.1;

        // Use a separate iterator to avoid modifying the original indices vector
        let filtered_indices: Vec<usize> = range.filter(|&i| !indices.contains(&i)).collect();

        // Extract the substring without the excluded indices
        let modified_substring: String = filtered_indices
            .iter()
            .map(|&i| self.json_string.chars().nth(i).unwrap())
            .collect();

        modified_substring
    }

    fn parse_null(&mut self) -> Result<JsonValue, &'static str> {
        let next_char = self.json_string.chars().nth(self.index);

        if next_char == Some('n') {
            self.index += 4; // Consume 'null'
            Ok(JsonValue::Null)
        } else {
            Err("Invalid null value")
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
}
