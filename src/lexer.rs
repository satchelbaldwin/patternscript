#[derive(Debug, PartialEq, Hash, Clone)]

pub enum Op {
    Test,
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Exp,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Condition {
    Unless,
    When,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Keyword {
    For,
    Wait,
    Spawn,
    Bullet,
    Path,
    Pattern,
    Let,
    Seconds,
    Frames,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Id(String),
    Number(String),
    String(String),
    OpenParen,
    CloseParen,
    OpenBlock,
    CloseBlock,
    Comma,
    Operator(Op),
    EOF,
    RangeSeparator,
    Assign,
    Semicolon,
    Keyword(Keyword),
    Condition(Condition),
    LexerError(char),
}

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

    pub fn lookahead(&mut self, n: u32) -> Option<Token> {
        let previous_cursor = self.cursor;
        let mut i = 0;
        while i < (n - 1) {
            let _ = self.next_token();
            i += 1;
        }
        let result = self.next_token();
        self.cursor = previous_cursor;
        result
    }

    pub fn next_token(&mut self) -> Option<Token> {
        match self.characters {
            Some(ref chars) => {
                if self.cursor >= chars.len() {
                    return Some(Token::EOF);
                }
                self.lookahead_cursor = self.cursor + 1;
                let initial = chars[self.cursor];

                let mut exact_match = |word: &str| -> bool {
                    let length: usize = word.len();

                    if self.cursor + length > chars.len() {
                        return false;
                    }

                    // things which can happen directly after a keyword and not be a part of it
                    let terminators = vec![';', ' ', '(', '{', '='];
                    // tokens which break the above rules
                    let special_tokens: Vec<&str> = vec!["//", "==", "..."];
                    // found is what we find in the character range of the expected word
                    let found: String = chars[self.cursor..self.cursor + length].iter().collect();
                    if word == found {
                        // ignore keywords that can have alphanumeric
                        if !special_tokens.contains(&word) {
                            // if it's the last token, nothing can come after that invalidates it
                            if self.cursor + length == chars.len() {
                                return true;
                            }
                            let lookahead = chars[self.cursor + length];
                            if !terminators.contains(&lookahead) {
                                // keyword has alphanumeric letters after -- this is not a keyword!
                                // see: origin where origin[0..2]='or'
                                return false;
                            }
                        }
                        self.cursor = self.cursor + length - 1;
                    }

                    word == found
                };

                let token: Token = match initial {
                    '(' => Token::OpenParen,
                    ')' => Token::CloseParen,
                    '{' => Token::OpenBlock,
                    '}' => Token::CloseBlock,
                    ',' => Token::Comma,
                    ';' => Token::Semicolon,
                    '+' => Token::Operator(Op::Add),
                    '-' => Token::Operator(Op::Sub),
                    '*' => Token::Operator(Op::Mul),
                    '^' => Token::Operator(Op::Exp),
                    _ if exact_match("==") => Token::Operator(Op::Test),
                    '=' => Token::Assign,
                    _ if exact_match("//") => {
                        while chars[self.lookahead_cursor] != '\n' {
                            self.lookahead_cursor += 1;
                        }
                        self.cursor = self.lookahead_cursor + 1;
                        return self.next_token();
                    }
                    '/' => Token::Operator(Op::Div),
                    _ if exact_match("for") => Token::Keyword(Keyword::For),
                    _ if exact_match("...") => Token::RangeSeparator,
                    _ if exact_match("and") => Token::Operator(Op::And),
                    _ if exact_match("or") => Token::Operator(Op::Or),
                    _ if exact_match("unless") => Token::Condition(Condition::Unless),
                    _ if exact_match("when") => Token::Condition(Condition::When),
                    _ if exact_match("wait") => Token::Keyword(Keyword::Wait),
                    _ if exact_match("spawn") => Token::Keyword(Keyword::Spawn),
                    _ if exact_match("bullet") => Token::Keyword(Keyword::Bullet),
                    _ if exact_match("path") => Token::Keyword(Keyword::Path),
                    _ if exact_match("pattern") => Token::Keyword(Keyword::Pattern),
                    _ if exact_match("let") => Token::Keyword(Keyword::Let),
                    _ if exact_match("seconds") => Token::Keyword(Keyword::Seconds),
                    _ if exact_match("frames") => Token::Keyword(Keyword::Frames),

                    number if initial.is_digit(10) => {
                        let mut full_number: String = String::new();
                        full_number.push(number); // first digit

                        self.lookahead_cursor = self.cursor + 1;
                        while chars[self.lookahead_cursor].is_digit(10) {
                            full_number.push(chars[self.lookahead_cursor]);
                            self.lookahead_cursor += 1;
                        }

                        if chars[self.lookahead_cursor] == '.' {
                            // handle case x...y where x. .. y is wrong
                            if chars[self.lookahead_cursor + 1] == '.' {
                                self.cursor = self.lookahead_cursor;
                                return Some(Token::Number(full_number));
                            }
                            full_number.push(chars[self.lookahead_cursor]);
                            self.lookahead_cursor += 1;
                            while chars[self.lookahead_cursor].is_digit(10) {
                                full_number.push(chars[self.lookahead_cursor]);
                                self.lookahead_cursor += 1;
                            }
                        }

                        self.cursor = self.lookahead_cursor - 1;
                        Token::Number(full_number)
                    }
                    '"' => {
                        let mut full_string: String = String::new();

                        self.lookahead_cursor = self.cursor + 1;
                        while chars[self.lookahead_cursor] != '"' {
                            full_string.push(chars[self.lookahead_cursor]);
                            // if backslash push next without checking loop condition to catch \"
                            if chars[self.lookahead_cursor] == '\\' {
                                self.lookahead_cursor += 1;
                                full_string.pop(); // remove \
                                full_string.push(chars[self.lookahead_cursor]);
                            }
                            self.lookahead_cursor += 1;
                        }
                        self.cursor = self.lookahead_cursor;
                        Token::String(full_string)
                    }
                    c if initial.is_ascii_alphabetic() => {
                        let mut full_id: String = String::new();
                        full_id.push(c); // first digit

                        let mut c = chars[self.lookahead_cursor];
                        while c.is_alphanumeric() || c == '_' || c == '-' {
                            full_id.push(chars[self.lookahead_cursor]);
                            self.lookahead_cursor += 1;
                            c = chars[self.lookahead_cursor];
                        }

                        self.cursor = self.lookahead_cursor - 1;

                        Token::Id(full_id)
                    }
                    _ if initial.is_ascii_whitespace() => {
                        self.cursor += 1;
                        return self.next_token();
                    }
                    c => Token::LexerError(c),
                };
                self.cursor += 1;
                Some(token)
            }
            None => None,
        }
    }
}
