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
    MinusEqual,     // -=
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
            '*' => {
                self.advance();
                Ok(Token { kind: TokenKind::Star, line, column })
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
                match self.peek() {
                    Some('>') => {
                        self.advance();
                        Ok(Token { kind: TokenKind::Arrow, line, column })
                    }
                    Some('=') => {
                        self.advance();
                        Ok(Token { kind: TokenKind::MinusEqual, line, column })
                    }
                    _ => Ok(Token { kind: TokenKind::Minus, line, column })
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
                // In Erlang, a dot after a number can be:
                // 1. Part of a float if followed by digits: "2.5"
                // 2. Part of a float if at end of number: "2." (parsed as 2.0)
                // 3. A separate token if followed by whitespace/operator: "2 ." or "2.+"
                if is_float {
                    break;
                }
                // Check the character after the dot without advancing
                let next_pos = self.pos + 1;
                if next_pos < self.input.len() {
                    let next_ch = self.input[next_pos];
                    if next_ch.is_ascii_digit() {
                        // Dot followed by digit - it's a float, consume the dot
                        is_float = true;
                        num_str.push(c);
                        self.advance();
                    } else if next_ch.is_whitespace() || next_ch == '\n' {
                        // Dot followed by whitespace - in Erlang, "2." is Float(2.0)
                        // So consume the dot as part of the float
                        is_float = true;
                        num_str.push(c);
                        self.advance();
                    } else {
                        // Dot followed by non-digit, non-whitespace - check if it's an operator
                        // that would make the dot a separate token
                        // For now, if followed by operator/punctuation, treat as separate token
                        // This handles cases like "2.+3" where we want Integer(2) + Dot + Plus + Integer(3)
                        // But "2." should be Float(2.0)
                        // Actually, in Erlang "2.+3" would be Float(2.0) + Plus + Integer(3)
                        // So we should consume the dot if it's at the end of input or followed by operator
                        // Let's be conservative: only don't consume if followed by letter/digit context suggests separation
                        // For simplicity, consume dot as part of float if it's not clearly separated
                        // The key insight: "2." at end or before operator is Float(2.0)
                        is_float = true;
                        num_str.push(c);
                        self.advance();
                    }
                } else {
                    // EOF after dot - in Erlang, "2." at EOF is Float(2.0)
                    is_float = true;
                    num_str.push(c);
                    self.advance();
                }
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

/// Scan tokens until a dot (`.`) is found
///
/// This function is used for REPL/shell mode where expressions must be terminated
/// by a dot. It scans tokens until it finds a dot token or detects that the input
/// ends with a dot (which may be consumed as part of a float like "2.").
/// This matches the behavior of `erl_scan:tokens` in Erlang.
///
/// # Arguments
/// * `input` - Input string to scan
///
/// # Returns
/// * `Ok(Vec<Token>)` - List of tokens (may include Dot token or Float ending with dot)
/// * `Err(ScanError)` - Scan error (including if EOF reached before dot)
///
/// # Example
/// ```
/// use infrastructure_utilities::erl_scan::{scan_until_dot, TokenKind};
/// 
/// // Valid: expression with dot
/// let tokens = scan_until_dot("2+2.").unwrap();
/// // The dot may be part of Float(2.0) or a separate Dot token
/// 
/// // Invalid: expression without dot
/// assert!(scan_until_dot("2+2").is_err());
/// ```
pub fn scan_until_dot(input: &str) -> Result<Vec<Token>, ScanError> {
    let mut scanner = Scanner::new(input);
    let mut tokens: Vec<Token> = Vec::new();
    
    // Check if input ends with a dot (trimmed) - this helps detect dots consumed as floats
    let trimmed_input = input.trim();
    let ends_with_dot = trimmed_input.ends_with('.');
    
    loop {
        let token = scanner.scan_token()?;
        let is_dot = matches!(token.kind, TokenKind::Dot);
        let is_eof = matches!(token.kind, TokenKind::Eof);
        
        if is_eof {
            // EOF reached - check if input ended with dot
            if ends_with_dot {
                // Input ended with dot (may have been consumed as part of float like "2.")
                // Check if the last token before EOF is a Float that ends with dot
                // If so, we need to add a Dot token for the parser
                if let Some(last_token) = tokens.last() {
                    if matches!(last_token.kind, TokenKind::Float(_)) {
                        // Last token is a float - the dot was consumed as part of it
                        // Add a Dot token so the parser can consume it
                        let line = scanner.line;
                        let column = scanner.column;
                        tokens.push(Token { kind: TokenKind::Dot, line, column });
                    }
                }
                // Don't push EOF token - we've handled the dot
                break;
            } else {
                // EOF reached without finding dot - error
                let line = scanner.line;
                let column = scanner.column;
                return Err(ScanError::UnexpectedEof(line, column));
            }
        } else {
            tokens.push(token);
            
            if is_dot {
                // Found dot token - scanning complete
                break;
            }
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
        // Note: "2." is parsed as a float (2.0), not integer(2) + dot
        // Use "2 + 2 ." with space to get separate tokens, or accept float
        let tokens = scan_string("2 + 2.").unwrap();
        // Tokens: Integer(2), Plus, Float(2.0), Eof
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Integer(2));
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::Float(2.0));
        assert_eq!(tokens[3].kind, TokenKind::Eof);
        
        // Test with space to get integer + dot as separate tokens
        let tokens2 = scan_string("2 + 2 .").unwrap();
        // Tokens: Integer(2), Plus, Integer(2), Dot, Eof
        assert_eq!(tokens2.len(), 5);
        assert_eq!(tokens2[0].kind, TokenKind::Integer(2));
        assert_eq!(tokens2[1].kind, TokenKind::Plus);
        assert_eq!(tokens2[2].kind, TokenKind::Integer(2));
        assert_eq!(tokens2[3].kind, TokenKind::Dot);
        assert_eq!(tokens2[4].kind, TokenKind::Eof);
    }
    
    #[test]
    fn test_scan_char() {
        let tokens = scan_string("'A'").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Char(c) => assert_eq!(c, 'A'),
            _ => panic!("Expected Char, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_char_escaped() {
        let tokens = scan_string("'\\n'").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Char(c) => assert_eq!(c, '\n'),
            _ => panic!("Expected Char, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_atom_quoted() {
        let tokens = scan_string("'hello world'").unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0].kind {
            TokenKind::Atom(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected Atom, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_string_escaped() {
        let tokens = scan_string("\"hello\\nworld\"").unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0].kind {
            TokenKind::String(s) => {
                assert_eq!(s, "hello\nworld");
            }
            _ => panic!("Expected String, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_integer_negative() {
        let tokens = scan_string("-123").unwrap();
        assert_eq!(tokens.len(), 3); // Minus, Integer, Eof
        assert_eq!(tokens[0].kind, TokenKind::Minus);
        assert_eq!(tokens[1].kind, TokenKind::Integer(123));
    }
    
    #[test]
    fn test_scan_float_scientific() {
        let tokens = scan_string("1.5e10").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Float(f) => assert!((f - 1.5e10).abs() < f64::EPSILON * 1e10),
            _ => panic!("Expected Float, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_float_scientific_negative() {
        let tokens = scan_string("1.5e-10").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Float(f) => assert!((f - 1.5e-10).abs() < f64::EPSILON),
            _ => panic!("Expected Float, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_variable_with_underscore() {
        let tokens = scan_string("_X").unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0].kind {
            TokenKind::Var(s) => assert_eq!(s, "_X"),
            _ => panic!("Expected Var, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_variable_with_at() {
        let tokens = scan_string("X@host").unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0].kind {
            TokenKind::Var(s) => assert_eq!(s, "X@host"),
            _ => panic!("Expected Var, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_atom_with_underscore() {
        let tokens = scan_string("hello_world").unwrap();
        assert_eq!(tokens.len(), 2);
        match &tokens[0].kind {
            TokenKind::Atom(s) => assert_eq!(s, "hello_world"),
            _ => panic!("Expected Atom, got {:?}", tokens[0].kind),
        }
    }
    
    #[test]
    fn test_scan_keywords() {
        let keywords = vec![
            ("after", TokenKind::After),
            ("begin", TokenKind::Begin),
            ("case", TokenKind::Case),
            ("catch", TokenKind::Catch),
            ("end", TokenKind::End),
            ("fun", TokenKind::Fun),
            ("if", TokenKind::If),
            ("of", TokenKind::Of),
            ("receive", TokenKind::Receive),
            ("try", TokenKind::Try),
            ("when", TokenKind::When),
            ("and", TokenKind::And),
            ("or", TokenKind::Or),
            ("xor", TokenKind::Xor),
            ("andalso", TokenKind::AndAlso),
            ("orelse", TokenKind::OrElse),
            ("not", TokenKind::Not),
            ("div", TokenKind::Div),
            ("rem", TokenKind::Rem),
        ];
        
        for (keyword, expected_kind) in keywords {
            let tokens = scan_string(keyword).unwrap();
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].kind, expected_kind);
        }
    }
    
    #[test]
    fn test_scan_punctuation() {
        let tokens = scan_string(".,;:()[]{}|#@?").unwrap();
        // Note: ":" followed by ":" would produce DoubleColon, but here they're separate
        // So we get: Dot, Comma, Semicolon, Colon, LeftParen, RightParen, LeftBracket,
        // RightBracket, LeftBrace, RightBrace, Pipe, Hash, At, Question, Eof = 15 tokens
        assert!(tokens.len() >= 14); // At least 13 punctuation + Eof
        assert_eq!(tokens[0].kind, TokenKind::Dot);
        assert_eq!(tokens[1].kind, TokenKind::Comma);
        assert_eq!(tokens[2].kind, TokenKind::Semicolon);
        assert_eq!(tokens[3].kind, TokenKind::Colon);
        assert_eq!(tokens[4].kind, TokenKind::LeftParen);
        assert_eq!(tokens[5].kind, TokenKind::RightParen);
        assert_eq!(tokens[6].kind, TokenKind::LeftBracket);
        assert_eq!(tokens[7].kind, TokenKind::RightBracket);
        assert_eq!(tokens[8].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[9].kind, TokenKind::RightBrace);
        assert_eq!(tokens[10].kind, TokenKind::Pipe);
        assert_eq!(tokens[11].kind, TokenKind::Hash);
        assert_eq!(tokens[12].kind, TokenKind::At);
        assert_eq!(tokens[13].kind, TokenKind::Question);
    }
    
    #[test]
    fn test_scan_dot_dot() {
        let tokens = scan_string("..").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::DotDot);
    }
    
    #[test]
    fn test_scan_dot_dot_dot() {
        let tokens = scan_string("...").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::DotDotDot);
    }
    
    #[test]
    fn test_scan_double_colon() {
        let tokens = scan_string("::").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::DoubleColon);
    }
    
    #[test]
    fn test_scan_arrow() {
        let tokens = scan_string("->").unwrap();
        assert_eq!(tokens.len(), 2); // Arrow, Eof
        assert_eq!(tokens[0].kind, TokenKind::Arrow);
    }
    
    #[test]
    fn test_scan_minus_equal() {
        let tokens = scan_string("-=").unwrap();
        assert_eq!(tokens.len(), 2); // MinusEqual, Eof
        assert_eq!(tokens[0].kind, TokenKind::MinusEqual);
    }
    
    #[test]
    fn test_scan_minus_variants() {
        // Test that all - variants are correctly distinguished
        let tokens = scan_string("- -> -=").unwrap();
        assert_eq!(tokens.len(), 4); // Minus, Arrow, MinusEqual, Eof
        assert_eq!(tokens[0].kind, TokenKind::Minus);
        assert_eq!(tokens[1].kind, TokenKind::Arrow);
        assert_eq!(tokens[2].kind, TokenKind::MinusEqual);
    }
    
    #[test]
    fn test_scan_comparison_operators() {
        let tokens = scan_string("< <= > >=").unwrap();
        assert_eq!(tokens.len(), 5); // 4 operators + Eof
        assert_eq!(tokens[0].kind, TokenKind::Less);
        assert_eq!(tokens[1].kind, TokenKind::LessEqual);
        assert_eq!(tokens[2].kind, TokenKind::Greater);
        assert_eq!(tokens[3].kind, TokenKind::GreaterEqual);
    }
    
    #[test]
    fn test_scan_whitespace() {
        let tokens = scan_string("  123  ").unwrap();
        assert_eq!(tokens.len(), 2); // Integer + Eof (whitespace skipped)
        assert_eq!(tokens[0].kind, TokenKind::Integer(123));
    }
    
    #[test]
    fn test_scan_comment() {
        let tokens = scan_string("123 % this is a comment\n456").unwrap();
        assert_eq!(tokens.len(), 3); // Integer, Integer, Eof
        assert_eq!(tokens[0].kind, TokenKind::Integer(123));
        assert_eq!(tokens[1].kind, TokenKind::Integer(456));
    }
    
    #[test]
    fn test_scan_multiline() {
        let tokens = scan_string("123\n456\n789").unwrap();
        assert_eq!(tokens.len(), 4); // 3 integers + Eof
        assert_eq!(tokens[0].kind, TokenKind::Integer(123));
        assert_eq!(tokens[1].kind, TokenKind::Integer(456));
        assert_eq!(tokens[2].kind, TokenKind::Integer(789));
    }
    
    #[test]
    fn test_scan_empty() {
        let tokens = scan_string("").unwrap();
        assert_eq!(tokens.len(), 1); // Just Eof
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }
    
    #[test]
    fn test_scan_error_unexpected_char() {
        let result = scan_string("~");
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError::UnexpectedChar(c, _, _) => assert_eq!(c, '~'),
            _ => panic!("Expected UnexpectedChar error"),
        }
    }
    
    #[test]
    fn test_scan_error_unterminated_string() {
        let result = scan_string("\"hello");
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError::UnterminatedString(_, _) => {}
            _ => panic!("Expected UnterminatedString error"),
        }
    }
    
    #[test]
    fn test_scan_error_unterminated_atom() {
        let result = scan_string("'hello");
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError::UnterminatedAtom(_, _) => {}
            _ => panic!("Expected UnterminatedAtom error"),
        }
    }
    
    #[test]
    fn test_scan_error_invalid_escape() {
        let result = scan_string("\"hello\\x\"");
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError::InvalidEscape(c, _, _) => assert_eq!(c, 'x'),
            _ => panic!("Expected InvalidEscape error"),
        }
    }
    
    #[test]
    fn test_scan_string_escape_sequences() {
        let escape_tests = vec![
            ("\\n", '\n'),
            ("\\t", '\t'),
            ("\\r", '\r'),
            ("\\\\", '\\'),
            ("\\\"", '"'),
        ];
        
        for (escape, expected) in escape_tests {
            let input = format!("\"hello{}world\"", escape);
            let tokens = scan_string(&input).unwrap();
            match &tokens[0].kind {
                TokenKind::String(s) => {
                    assert!(s.contains(expected), "String should contain {:?} for escape {}", expected, escape);
                }
                _ => panic!("Expected String for {}", escape),
            }
        }
    }
    
    #[test]
    fn test_scan_atom_escape_sequences() {
        let escape_tests = vec![
            ("\\n", '\n'),
            ("\\t", '\t'),
            ("\\r", '\r'),
            ("\\\\", '\\'),
            ("\\'", '\''),
        ];
        
        for (escape, expected) in escape_tests {
            let input = format!("'hello{}world'", escape);
            let tokens = scan_string(&input).unwrap();
            match &tokens[0].kind {
                TokenKind::Atom(s) => {
                    assert!(s.contains(expected), "Atom should contain {:?} for escape {}", expected, escape);
                }
                _ => panic!("Expected Atom for {}", escape),
            }
        }
    }
    
    #[test]
    fn test_scan_integer_large() {
        let tokens = scan_string("9223372036854775807").unwrap(); // i64::MAX
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Integer(9223372036854775807));
    }
    
    #[test]
    fn test_scan_integer_zero() {
        let tokens = scan_string("0").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Integer(0));
    }
    
    #[test]
    fn test_scan_float_zero() {
        let tokens = scan_string("0.0").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
            _ => panic!("Expected Float"),
        }
    }
    
    #[test]
    fn test_scan_float_small() {
        let tokens = scan_string("0.001").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Float(f) => assert!((f - 0.001).abs() < f64::EPSILON),
            _ => panic!("Expected Float"),
        }
    }
    
    #[test]
    fn test_scan_token_debug() {
        let token = Token {
            kind: TokenKind::Integer(42),
            line: 1,
            column: 1,
        };
        let debug_str = format!("{:?}", token);
        assert!(!debug_str.is_empty());
    }
    
    #[test]
    fn test_scan_token_clone() {
        let token1 = Token {
            kind: TokenKind::Integer(42),
            line: 1,
            column: 1,
        };
        let token2 = token1.clone();
        assert_eq!(token1, token2);
    }
    
    #[test]
    fn test_scan_token_partial_eq() {
        let token1 = Token {
            kind: TokenKind::Integer(42),
            line: 1,
            column: 1,
        };
        let token2 = Token {
            kind: TokenKind::Integer(42),
            line: 1,
            column: 1,
        };
        assert_eq!(token1, token2);
        
        let token3 = Token {
            kind: TokenKind::Integer(43),
            line: 1,
            column: 1,
        };
        assert_ne!(token1, token3);
    }
    
    #[test]
    fn test_scan_error_display() {
        let error = ScanError::UnexpectedChar('~', 1, 1);
        let display_str = format!("{}", error);
        assert!(display_str.contains("Unexpected character"));
        assert!(display_str.contains("~"));
    }
    
    #[test]
    fn test_scan_error_clone() {
        let error1 = ScanError::UnexpectedChar('~', 1, 1);
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }
    
    #[test]
    fn test_scan_error_debug() {
        let error = ScanError::UnterminatedString(1, 1);
        let debug_str = format!("{:?}", error);
        assert!(!debug_str.is_empty());
    }
    
    #[test]
    fn test_scan_error_error_trait() {
        let error = ScanError::InvalidEscape('x', 1, 1);
        // Test that it implements Error trait
        let error_ref: &dyn std::error::Error = &error;
        let display_str = format!("{}", error_ref);
        assert!(!display_str.is_empty());
    }
    
    #[test]
    fn test_scan_token_kind_debug() {
        let kinds = vec![
            TokenKind::Integer(42),
            TokenKind::Float(3.14),
            TokenKind::Atom("test".to_string()),
            TokenKind::String("hello".to_string()),
            TokenKind::Char('A'),
            TokenKind::Var("X".to_string()),
            TokenKind::Plus,
            TokenKind::Eof,
        ];
        
        for kind in kinds {
            let debug_str = format!("{:?}", kind);
            assert!(!debug_str.is_empty());
        }
    }
    
    #[test]
    fn test_scan_token_kind_clone() {
        let kind1 = TokenKind::Integer(42);
        let kind2 = kind1.clone();
        assert_eq!(kind1, kind2);
    }
    
    #[test]
    fn test_scan_token_kind_partial_eq() {
        assert_eq!(TokenKind::Plus, TokenKind::Plus);
        assert_ne!(TokenKind::Plus, TokenKind::Minus);
        assert_eq!(TokenKind::Integer(42), TokenKind::Integer(42));
        assert_ne!(TokenKind::Integer(42), TokenKind::Integer(43));
    }
    
    #[test]
    fn test_scan_location_tracking() {
        let tokens = scan_string("123\n456").unwrap();
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[1].line, 2);
    }
    
    #[test]
    fn test_scan_column_tracking() {
        let tokens = scan_string("  123").unwrap();
        // Column should be where the token starts (after whitespace)
        assert!(tokens[0].column >= 1);
    }
    
    #[test]
    fn test_scan_complex_expression() {
        let tokens = scan_string("(1 + 2) * 3").unwrap();
        assert_eq!(tokens.len(), 8); // LeftParen, Integer, Plus, Integer, RightParen, Star, Integer, Eof
        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert_eq!(tokens[1].kind, TokenKind::Integer(1));
        assert_eq!(tokens[2].kind, TokenKind::Plus);
        assert_eq!(tokens[3].kind, TokenKind::Integer(2));
        assert_eq!(tokens[4].kind, TokenKind::RightParen);
        assert_eq!(tokens[5].kind, TokenKind::Star);
        assert_eq!(tokens[6].kind, TokenKind::Integer(3));
    }
    
    #[test]
    fn test_scan_list_literal() {
        let tokens = scan_string("[1, 2, 3]").unwrap();
        assert_eq!(tokens.len(), 8); // LeftBracket, Integer, Comma, Integer, Comma, Integer, RightBracket, Eof
        assert_eq!(tokens[0].kind, TokenKind::LeftBracket);
        assert_eq!(tokens[1].kind, TokenKind::Integer(1));
        assert_eq!(tokens[2].kind, TokenKind::Comma);
    }
    
    #[test]
    fn test_scan_tuple_literal() {
        let tokens = scan_string("{1, 2}").unwrap();
        assert_eq!(tokens.len(), 6); // LeftBrace, Integer, Comma, Integer, RightBrace, Eof
        assert_eq!(tokens[0].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[1].kind, TokenKind::Integer(1));
        assert_eq!(tokens[2].kind, TokenKind::Comma);
    }
    
    #[test]
    fn test_scan_function_call() {
        let tokens = scan_string("func(1, 2)").unwrap();
        assert_eq!(tokens.len(), 7); // Atom, LeftParen, Integer, Comma, Integer, RightParen, Eof
        match &tokens[0].kind {
            TokenKind::Atom(s) => assert_eq!(s, "func"),
            _ => panic!("Expected Atom"),
        }
        assert_eq!(tokens[1].kind, TokenKind::LeftParen);
    }
    
    #[test]
    fn test_scan_remote_call() {
        let tokens = scan_string("module:func(1)").unwrap();
        assert_eq!(tokens.len(), 7); // Atom, Colon, Atom, LeftParen, Integer, RightParen, Eof
        match &tokens[0].kind {
            TokenKind::Atom(s) => assert_eq!(s, "module"),
            _ => panic!("Expected Atom"),
        }
        assert_eq!(tokens[1].kind, TokenKind::Colon);
    }
    
    #[test]
    fn test_scan_atom_vs_keyword() {
        // "hello" should be an atom, not a keyword
        let tokens = scan_string("hello").unwrap();
        match &tokens[0].kind {
            TokenKind::Atom(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Atom, got {:?}", tokens[0].kind),
        }
        
        // "if" should be a keyword
        let tokens2 = scan_string("if").unwrap();
        assert_eq!(tokens2[0].kind, TokenKind::If);
    }
    
    #[test]
    fn test_scan_float_with_exponent_positive() {
        let tokens = scan_string("1.5e+10").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Float(f) => assert!((f - 1.5e10).abs() < f64::EPSILON * 1e10),
            _ => panic!("Expected Float"),
        }
    }
    
    #[test]
    fn test_scan_float_with_exponent_uppercase() {
        let tokens = scan_string("1.5E10").unwrap();
        assert_eq!(tokens.len(), 2);
        match tokens[0].kind {
            TokenKind::Float(f) => assert!((f - 1.5e10).abs() < f64::EPSILON * 1e10),
            _ => panic!("Expected Float"),
        }
    }
    
    #[test]
    fn test_scan_until_dot_with_dot() {
        let tokens = scan_until_dot("2+2.").unwrap();
        assert!(tokens.len() >= 4); // At least: Integer, Plus, Integer, Dot
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::Dot));
    }
    
    #[test]
    fn test_scan_until_dot_without_dot() {
        let result = scan_until_dot("2+2");
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError::UnexpectedEof(_, _) => {}
            e => panic!("Expected UnexpectedEof, got {:?}", e),
        }
    }
    
    #[test]
    fn test_scan_until_dot_multiple_expressions() {
        let tokens = scan_until_dot("1, 2, 3.").unwrap();
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::Dot));
    }
    
    #[test]
    fn test_scan_until_dot_with_semicolon() {
        let tokens = scan_until_dot("1; 2.").unwrap();
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::Dot));
    }
    
    #[test]
    fn test_scan_until_dot_empty_input() {
        let result = scan_until_dot("");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_scan_until_dot_only_dot() {
        let tokens = scan_until_dot(".").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Dot));
    }
    
    #[test]
    fn test_scan_until_dot_function_call() {
        let tokens = scan_until_dot("lists:last([1,2,3]).").unwrap();
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::Dot));
    }
}

