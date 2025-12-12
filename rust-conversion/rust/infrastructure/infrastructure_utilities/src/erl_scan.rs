//! Erlang Scanner (erl_scan equivalent)
//!
//! Tokenizes Erlang source code into tokens. This is the first step in parsing
//! Erlang expressions. Based on erl_scan.erl from lib/stdlib.

use entities_data_handling::AtomEncoding;

/// Token type with location information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token kind
    pub kind: TokenKind,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// Token kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    Atom(String),
    String(String),
    Char(char),
    
    // Variables
    Var(String),
    
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Div,            // div
    Rem,            // rem
    Bang,           // !
    Equal,          // =
    EqualEqual,     // ==
    NotEqual,       // /=
    Less,           // <
    LessEqual,      // =<
    Greater,        // >
    GreaterEqual,   // >=
    And,            // and
    Or,             // or
    Xor,            // xor
    AndAlso,        // andalso
    OrElse,         // orelse
    Not,            // not
    
    // Punctuation
    Dot,            // .
    Comma,          // ,
    Semicolon,      // ;
    Colon,          // :
    DoubleColon,    // ::
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    LeftBrace,      // {
    RightBrace,     // }
    Pipe,           // |
    Hash,           // #
    At,             // @
    Question,       // ?
    Arrow,          // ->
    DotDot,         // ..
    DotDotDot,      // ...
    
    // Keywords
    After,          // after
    Begin,          // begin
    Case,           // case
    Catch,          // catch
    End,            // end
    Fun,            // fun
    If,             // if
    Of,             // of
    Receive,        // receive
    Try,            // try
    When,           // when
    
    // Special
    Eof,            // End of file
    Error(String), // Error token
}

/// Scanner error
#[derive(Debug, Clone, PartialEq)]
pub enum ScanError {
    UnexpectedChar(char, usize, usize),
    UnterminatedString(usize, usize),
    UnterminatedAtom(usize, usize),
    InvalidEscape(char, usize, usize),
    InvalidNumber(String, usize, usize),
    UnexpectedEof(usize, usize),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScanError::UnexpectedChar(c, line, col) => {
                write!(f, "Unexpected character '{}' at line {}, column {}", c, line, col)
            }
            ScanError::UnterminatedString(line, col) => {
                write!(f, "Unterminated string at line {}, column {}", line, col)
            }
            ScanError::UnterminatedAtom(line, col) => {
                write!(f, "Unterminated atom at line {}, column {}", line, col)
            }
            ScanError::InvalidEscape(c, line, col) => {
                write!(f, "Invalid escape sequence '\\{}' at line {}, column {}", c, line, col)
            }
            ScanError::InvalidNumber(s, line, col) => {
                write!(f, "Invalid number '{}' at line {}, column {}", s, line, col)
            }
            ScanError::UnexpectedEof(line, col) => {
                write!(f, "Unexpected end of file at line {}, column {}", line, col)
            }
        }
    }
}

impl std::error::Error for ScanError {}

/// Scanner state
struct Scanner {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Scanner {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '%' {
                // Comment - skip to end of line
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }
    
    fn scan_token(&mut self) -> Result<Token, ScanError> {
        self.skip_whitespace();
        
        let line = self.line;
        let column = self.column;
        
        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(Token { kind: TokenKind::Eof, line, column }),
        };
        
        match ch {
            '+' => {
                self.advance();
                Ok(Token { kind: TokenKind::Plus, line, column })
            }
            '-' => {
                self.advance();
                Ok(Token { kind: TokenKind::Minus, line, column })
            }
            '*' => {
                self.advance();
                Ok(Token { kind: TokenKind::Star, line, column })
            }
            '/' => {
                self.advance();
                Ok(Token { kind: TokenKind::Slash, line, column })
            }
            '!' => {
                self.advance();
                Ok(Token { kind: TokenKind::Bang, line, column })
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::EqualEqual, line, column })
                } else {
                    Ok(Token { kind: TokenKind::Equal, line, column })
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::LessEqual, line, column })
                } else {
                    Ok(Token { kind: TokenKind::Less, line, column })
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::GreaterEqual, line, column })
                } else {
                    Ok(Token { kind: TokenKind::Greater, line, column })
                }
            }
            '/' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token { kind: TokenKind::NotEqual, line, column })
                } else {
                    Ok(Token { kind: TokenKind::Slash, line, column })
                }
            }
            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        Ok(Token { kind: TokenKind::DotDotDot, line, column })
                    } else {
                        Ok(Token { kind: TokenKind::DotDot, line, column })
                    }
                } else {
                    Ok(Token { kind: TokenKind::Dot, line, column })
                }
            }
            ',' => {
                self.advance();
                Ok(Token { kind: TokenKind::Comma, line, column })
            }
            ';' => {
                self.advance();
                Ok(Token { kind: TokenKind::Semicolon, line, column })
            }
            ':' => {
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    Ok(Token { kind: TokenKind::DoubleColon, line, column })
                } else {
                    Ok(Token { kind: TokenKind::Colon, line, column })
                }
            }
            '(' => {
                self.advance();
                Ok(Token { kind: TokenKind::LeftParen, line, column })
            }
            ')' => {
                self.advance();
                Ok(Token { kind: TokenKind::RightParen, line, column })
            }
            '[' => {
                self.advance();
                Ok(Token { kind: TokenKind::LeftBracket, line, column })
            }
            ']' => {
                self.advance();
                Ok(Token { kind: TokenKind::RightBracket, line, column })
            }
            '{' => {
                self.advance();
                Ok(Token { kind: TokenKind::LeftBrace, line, column })
            }
            '}' => {
                self.advance();
                Ok(Token { kind: TokenKind::RightBrace, line, column })
            }
            '|' => {
                self.advance();
                Ok(Token { kind: TokenKind::Pipe, line, column })
            }
            '#' => {
                self.advance();
                Ok(Token { kind: TokenKind::Hash, line, column })
            }
            '@' => {
                self.advance();
                Ok(Token { kind: TokenKind::At, line, column })
            }
            '?' => {
                self.advance();
                Ok(Token { kind: TokenKind::Question, line, column })
            }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(Token { kind: TokenKind::Arrow, line, column })
                } else {
                    Ok(Token { kind: TokenKind::Minus, line, column })
                }
            }
            '\'' => {
                self.scan_atom_or_char(line, column)
            }
            '"' => {
                self.scan_string(line, column)
            }
            c if c.is_ascii_digit() => {
                self.scan_number(line, column)
            }
            c if c.is_ascii_uppercase() || c == '_' => {
                self.scan_variable(line, column)
            }
            c if c.is_ascii_lowercase() => {
                self.scan_atom_or_keyword(line, column)
            }
            _ => {
                let ch = self.advance().unwrap();
                Err(ScanError::UnexpectedChar(ch, line, column))
            }
        }
    }
    
    fn scan_atom_or_char(&mut self, line: usize, column: usize) -> Result<Token, ScanError> {
        self.advance(); // Skip opening quote
        let mut chars = Vec::new();
        let mut escaped = false;
        
        loop {
            match self.peek() {
                None => {
                    return Err(ScanError::UnterminatedAtom(line, column));
                }
                Some('\'') if !escaped => {
                    self.advance(); // Skip closing quote
                    let atom_str: String = chars.into_iter().collect();
                    // Check if it's a single character (char literal)
                    if atom_str.len() == 1 && !escaped {
                        return Ok(Token { kind: TokenKind::Char(atom_str.chars().next().unwrap()), line, column });
                    }
                    return Ok(Token { kind: TokenKind::Atom(atom_str), line, column });
                }
                Some('\\') if !escaped => {
                    escaped = true;
                    self.advance();
                }
                Some(c) => {
                    if escaped {
                        match c {
                            'n' => chars.push('\n'),
                            't' => chars.push('\t'),
                            'r' => chars.push('\r'),
                            '\\' => chars.push('\\'),
                            '\'' => chars.push('\''),
                            _ => return Err(ScanError::InvalidEscape(c, self.line, self.column)),
                        }
                        escaped = false;
                    } else {
                        chars.push(c);
                    }
                    self.advance();
                }
            }
        }
    }
    
    fn scan_string(&mut self, line: usize, column: usize) -> Result<Token, ScanError> {
        self.advance(); // Skip opening quote
        let mut chars = Vec::new();
        let mut escaped = false;
        
        loop {
            match self.peek() {
                None => {
                    return Err(ScanError::UnterminatedString(line, column));
                }
                Some('"') if !escaped => {
                    self.advance(); // Skip closing quote
                    let s: String = chars.into_iter().collect();
                    return Ok(Token { kind: TokenKind::String(s), line, column });
                }
                Some('\\') if !escaped => {
                    escaped = true;
                    self.advance();
                }
                Some(c) => {
                    if escaped {
                        match c {
                            'n' => chars.push('\n'),
                            't' => chars.push('\t'),
                            'r' => chars.push('\r'),
                            '\\' => chars.push('\\'),
                            '"' => chars.push('"'),
                            _ => return Err(ScanError::InvalidEscape(c, self.line, self.column)),
                        }
                        escaped = false;
                    } else {
                        chars.push(c);
                    }
                    self.advance();
                }
            }
        }
    }
    
    fn scan_number(&mut self, line: usize, column: usize) -> Result<Token, ScanError> {
        let mut num_str = String::new();
        let mut is_float = false;
        
        // Scan integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' {
                if is_float {
                    break;
                }
                is_float = true;
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Scan fractional part
        if is_float {
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        // Scan exponent
        if let Some('e') | Some('E') = self.peek() {
            num_str.push(self.advance().unwrap());
            if let Some('+') | Some('-') = self.peek() {
                num_str.push(self.advance().unwrap());
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        if is_float {
            match num_str.parse::<f64>() {
                Ok(f) => Ok(Token { kind: TokenKind::Float(f), line, column }),
                Err(_) => Err(ScanError::InvalidNumber(num_str, line, column)),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(i) => Ok(Token { kind: TokenKind::Integer(i), line, column }),
                Err(_) => Err(ScanError::InvalidNumber(num_str, line, column)),
            }
        }
    }
    
    fn scan_variable(&mut self, line: usize, column: usize) -> Result<Token, ScanError> {
        let mut var_str = String::new();
        
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '@' {
                var_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(Token { kind: TokenKind::Var(var_str), line, column })
    }
    
    fn scan_atom_or_keyword(&mut self, line: usize, column: usize) -> Result<Token, ScanError> {
        let mut atom_str = String::new();
        
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '@' {
                atom_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check if it's a keyword
        let kind = match atom_str.as_str() {
            "after" => TokenKind::After,
            "begin" => TokenKind::Begin,
            "case" => TokenKind::Case,
            "catch" => TokenKind::Catch,
            "end" => TokenKind::End,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "of" => TokenKind::Of,
            "receive" => TokenKind::Receive,
            "try" => TokenKind::Try,
            "when" => TokenKind::When,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "xor" => TokenKind::Xor,
            "andalso" => TokenKind::AndAlso,
            "orelse" => TokenKind::OrElse,
            "not" => TokenKind::Not,
            "div" => TokenKind::Div,
            "rem" => TokenKind::Rem,
            _ => TokenKind::Atom(atom_str),
        };
        
        Ok(Token { kind, line, column })
    }
}

/// Scan a string into tokens
///
/// This is the main entry point for the scanner. It tokenizes Erlang source code.
///
/// # Arguments
/// * `input` - Erlang source code string
///
/// # Returns
/// * `Ok(Vec<Token>)` - List of tokens
/// * `Err(ScanError)` - Scan error
pub fn scan_string(input: &str) -> Result<Vec<Token>, ScanError> {
    let mut scanner = Scanner::new(input);
    let mut tokens = Vec::new();
    
    loop {
        let token = scanner.scan_token()?;
        let is_eof = matches!(token.kind, TokenKind::Eof);
        tokens.push(token);
        if is_eof {
            break;
        }
    }
    
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scan_integer() {
        let tokens = scan_string("123").unwrap();
        assert_eq!(tokens.len(), 2); // Integer + Eof
        assert_eq!(tokens[0].kind, TokenKind::Integer(123));
    }
    
    #[test]
    fn test_scan_float() {
        let tokens = scan_string("123.45").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Float(123.45));
    }
    
    #[test]
    fn test_scan_atom() {
        let tokens = scan_string("hello").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Atom("hello".to_string()));
    }
    
    #[test]
    fn test_scan_string() {
        let tokens = scan_string("\"hello\"").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
    }
    
    #[test]
    fn test_scan_variable() {
        let tokens = scan_string("X").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Var("X".to_string()));
    }
    
    #[test]
    fn test_scan_operators() {
        let tokens = scan_string("+ - * / = ==").unwrap();
        assert_eq!(tokens.len(), 7); // 6 operators + Eof
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Equal);
        assert_eq!(tokens[5].kind, TokenKind::EqualEqual);
    }
    
    #[test]
    fn test_scan_expression() {
        let tokens = scan_string("2 + 2.").unwrap();
        assert_eq!(tokens.len(), 5); // 2, +, 2, ., Eof
        assert_eq!(tokens[0].kind, TokenKind::Integer(2));
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::Integer(2));
        assert_eq!(tokens[3].kind, TokenKind::Dot);
    }
}

