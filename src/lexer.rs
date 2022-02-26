use ordered_float::NotNaN;

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Token {
    BeginArray,     // [
    BeginObject,    // {
    EndArray,       // ]
    EndObject,      // }
    NameSeparator,  // :
    ValueSeparator, // ,
    Number(NotNaN<f64>),
    NumberMatch, // Not a real number, but in place of "Any number" in the table
    String(String),
    StringMatch, // Not a real string, but in place of "Any string" in the table
    LexerError(char),
    True,
    False,
    Null,
    EOF,
}

impl Eq for Token {}

pub struct Lexer {
    source: String,
    characters: Option<Vec<char>>,
    cursor: usize,
    lookahead_cursor: usize,
}

impl Lexer {
    pub fn new(source: String) -> Lexer {
        let mut lexer = Lexer {
            source,
            characters: None,
            cursor: 0,
            lookahead_cursor: 0,
        };
        lexer.characters = Some(lexer.source.chars().collect());
        lexer
    }

    pub fn next_token(&mut self, shift: bool) -> Option<Token> {
        //println!("next token: {}", shift);
        match self.characters {
            Some(ref chars) => {
                if self.cursor >= chars.len() {
                    return Some(Token::EOF);
                }
                let initial = chars[self.cursor];
                let token: Token = match initial {
                    '[' => Token::BeginArray,
                    '{' => Token::BeginObject,
                    ']' => Token::EndArray,
                    '}' => Token::EndObject,
                    ':' => Token::NameSeparator,
                    ',' => Token::ValueSeparator,
                    number if initial.is_digit(10) => {
                        let mut full_number: String = String::new();
                        full_number.push(number); // first digit

                        self.lookahead_cursor = self.cursor + 1;
                        while chars[self.lookahead_cursor].is_digit(10) {
                            full_number.push(chars[self.lookahead_cursor]);
                            self.lookahead_cursor = self.lookahead_cursor + 1;
                        }

                        if shift {
                            self.cursor = self.lookahead_cursor - 1;
                        }

                        Token::Number(NotNaN::new(full_number.parse::<f64>().unwrap()).unwrap())
                    }
                    '"' => {
                        let mut full_string: String = String::new();

                        self.lookahead_cursor = self.cursor + 1;
                        while chars[self.lookahead_cursor] != '"' {
                            full_string.push(chars[self.lookahead_cursor]);
                            // if backslash push next without checking loop condition to catch \"
                            if chars[self.lookahead_cursor] == '\\' {
                                self.lookahead_cursor = self.lookahead_cursor + 1;
                                full_string.pop(); // remove \
                                full_string.push(chars[self.lookahead_cursor]);
                            }
                            self.lookahead_cursor = self.lookahead_cursor + 1;
                        }

                        if shift {
                            self.cursor = self.lookahead_cursor;
                        }

                        Token::String(full_string)
                    }
                    _ if initial.is_ascii_alphabetic() => {
                        let mut is_word = |word: &str| -> bool {
                            let length: usize = word.len();
                            let found: String =
                                chars[self.cursor..self.cursor + length].iter().collect();
                            if word == found {
                                if shift {
                                    self.cursor = self.cursor + length;
                                }
                            }
                            word == found
                        };
                        if is_word("true") {
                            return Some(Token::True);
                        }
                        if is_word("false") {
                            return Some(Token::False);
                        }
                        if is_word("null") {
                            return Some(Token::Null);
                        }
                        return Some(Token::LexerError(initial));
                    }
                    _ if initial.is_ascii_whitespace() => {
                        self.cursor = self.cursor + 1;
                        return self.next_token(shift);
                    }
                    c => {
                        if shift {
                            self.cursor = self.cursor + 1;
                        }
                        Token::LexerError(c)
                    }
                };
                if shift {
                    self.cursor = self.cursor + 1;
                }
                return Some(token);
            }
            None => None,
        }
    }
}
