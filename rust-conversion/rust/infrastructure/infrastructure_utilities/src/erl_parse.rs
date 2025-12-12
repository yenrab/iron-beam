//! Erlang Parser (erl_parse equivalent)
//!
//! Parses tokens into abstract syntax trees (AST). This is the second step in
//! parsing Erlang expressions, after tokenization by erl_scan.
//! Based on erl_parse.yrl from lib/stdlib.

use super::erl_scan::{Token, TokenKind, ScanError};
use std::fmt;

/// Abstract syntax tree node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    Integer(i64),
    /// Float literal
    Float(f64),
    /// Atom literal
    Atom(String),
    /// String literal
    String(String),
    /// Character literal
    Char(char),
    /// Variable
    Var(String),
    /// Nil (empty list)
    Nil,
    /// List construction [Head | Tail]
    Cons {
        head: Box<Expr>,
        tail: Box<Expr>,
    },
    /// List literal [E1, E2, ...]
    List(Vec<Expr>),
    /// Tuple {E1, E2, ...}
    Tuple(Vec<Expr>),
    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation
    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    /// Function call: Module:Function(Args)
    Call {
        module: Option<String>,
        function: String,
        args: Vec<Expr>,
    },
    /// Function call: Function(Args) (local)
    LocalCall {
        function: String,
        args: Vec<Expr>,
    },
    /// Parenthesized expression
    Paren(Box<Expr>),
    /// Pattern matching: Left = Right
    Match {
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    IntDiv,     // div
    Rem,        // rem
    Equal,      // ==
    NotEqual,   // /=
    Less,       // <
    LessEqual,  // =<
    Greater,    // >
    GreaterEqual, // >=
    And,        // and
    Or,         // or
    Xor,        // xor
    AndAlso,    // andalso
    OrElse,     // orelse
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Not,        // not
    Neg,        // - (unary minus)
    Pos,        // + (unary plus)
}

/// Parse error
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnexpectedEof,
    ExpectedToken(TokenKind, Token),
    InvalidExpression(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken(tok) => {
                write!(f, "Unexpected token {:?} at line {}, column {}", tok.kind, tok.line, tok.column)
            }
            ParseError::UnexpectedEof => {
                write!(f, "Unexpected end of file")
            }
            ParseError::ExpectedToken(expected, found) => {
                write!(f, "Expected {:?}, found {:?} at line {}, column {}", expected, found.kind, found.line, found.column)
            }
            ParseError::InvalidExpression(msg) => {
                write!(f, "Invalid expression: {}", msg)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parser state
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    
    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }
    
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        match self.peek() {
            Some(tok) if tok.kind == kind => {
                Ok(self.advance().unwrap())
            }
            Some(tok) => Err(ParseError::ExpectedToken(kind, tok.clone())),
            None => Err(ParseError::UnexpectedEof),
        }
    }
    
    fn parse_exprs(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = Vec::new();
        
        loop {
            if self.peek().map(|t| matches!(t.kind, TokenKind::Eof | TokenKind::Dot)).unwrap_or(true) {
                break;
            }
            exprs.push(self.parse_expr()?);
            
            if self.peek().map(|t| t.kind == TokenKind::Comma).unwrap_or(false) {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(exprs)
    }
    
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // Pattern matching has lowest precedence
        let left = self.parse_or_expr()?;
        
        // Check for pattern matching operator `=`
        if let Some(tok) = self.peek() {
            if tok.kind == TokenKind::Equal {
                self.advance(); // Skip `=`
                let right = self.parse_expr()?; // Recursively parse right side
                return Ok(Expr::Match {
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
        }
        
        Ok(left)
    }
    
    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        
        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::OrElse => {
                    self.advance();
                    let right = self.parse_and_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::OrElse,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Or => {
                    self.advance();
                    let right = self.parse_and_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Or,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }
    
    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comp_expr()?;
        
        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::AndAlso => {
                    self.advance();
                    let right = self.parse_comp_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::AndAlso,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::And => {
                    self.advance();
                    let right = self.parse_comp_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::And,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }
    
    fn parse_comp_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_add_expr()?;
        
        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::EqualEqual => {
                    self.advance();
                    let right = self.parse_add_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Equal,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::NotEqual => {
                    self.advance();
                    let right = self.parse_add_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::NotEqual,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Less => {
                    self.advance();
                    let right = self.parse_add_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Less,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::LessEqual => {
                    self.advance();
                    let right = self.parse_add_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::LessEqual,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Greater => {
                    self.advance();
                    let right = self.parse_add_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Greater,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::GreaterEqual => {
                    self.advance();
                    let right = self.parse_add_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::GreaterEqual,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }
    
    fn parse_add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_expr()?;
        
        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Plus => {
                    self.advance();
                    let right = self.parse_mul_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Add,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Minus => {
                    self.advance();
                    let right = self.parse_mul_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Sub,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }
    
    fn parse_mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expr()?;
        
        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Star => {
                    self.advance();
                    let right = self.parse_unary_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Mul,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Slash => {
                    self.advance();
                    let right = self.parse_unary_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Div,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Div => {
                    self.advance();
                    let right = self.parse_unary_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::IntDiv,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                TokenKind::Rem => {
                    self.advance();
                    let right = self.parse_unary_expr()?;
                    left = Expr::BinOp {
                        op: BinOp::Rem,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        Ok(left)
    }
    
    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(tok) => match tok.kind {
                TokenKind::Not => {
                    self.advance();
                    let expr = self.parse_unary_expr()?;
                    Ok(Expr::UnOp {
                        op: UnOp::Not,
                        expr: Box::new(expr),
                    })
                }
                TokenKind::Minus => {
                    self.advance();
                    let expr = self.parse_unary_expr()?;
                    Ok(Expr::UnOp {
                        op: UnOp::Neg,
                        expr: Box::new(expr),
                    })
                }
                TokenKind::Plus => {
                    self.advance();
                    let expr = self.parse_unary_expr()?;
                    Ok(Expr::UnOp {
                        op: UnOp::Pos,
                        expr: Box::new(expr),
                    })
                }
                _ => self.parse_primary_expr(),
            },
            None => Err(ParseError::UnexpectedEof),
        }
    }
    
    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek().ok_or(ParseError::UnexpectedEof)?.clone();
        match tok.kind {
            TokenKind::Integer(i) => {
                self.advance();
                Ok(Expr::Integer(i))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expr::Float(f))
            }
            TokenKind::Atom(s) => {
                self.advance();
                let atom_name = s.clone();
                // Check if it's a function call
                if self.peek().map(|t| t.kind == TokenKind::LeftParen).unwrap_or(false) {
                    self.parse_call(None, Some(atom_name))
                } else if self.peek().map(|t| t.kind == TokenKind::Colon).unwrap_or(false) {
                    // Module:Function call
                    let module_name = atom_name.clone();
                    let module = Some(module_name);
                    self.advance(); // Skip colon
                    if let Some(Token { kind: TokenKind::Atom(f), .. }) = self.peek() {
                        let func = f.clone();
                        self.advance();
                        if self.peek().map(|t| t.kind == TokenKind::LeftParen).unwrap_or(false) {
                            self.parse_call(module, Some(func))
                        } else {
                            Ok(Expr::Atom(atom_name))
                        }
                    } else {
                        Ok(Expr::Atom(atom_name))
                    }
                } else {
                    Ok(Expr::Atom(atom_name))
                }
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(Expr::String(s.clone()))
            }
            TokenKind::Char(c) => {
                self.advance();
                Ok(Expr::Char(c))
            }
            TokenKind::Var(v) => {
                self.advance();
                let var_name = v.clone();
                // Check if it's a function call
                if self.peek().map(|t| t.kind == TokenKind::LeftParen).unwrap_or(false) {
                    self.parse_call(None, Some(var_name))
                } else {
                    Ok(Expr::Var(var_name))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RightParen)?;
                Ok(Expr::Paren(Box::new(expr)))
            }
            TokenKind::LeftBracket => {
                self.parse_list()
            }
            TokenKind::LeftBrace => {
                self.parse_tuple()
            }
            _ => Err(ParseError::UnexpectedToken(tok)),
        }
    }
    
    fn parse_call(&mut self, module: Option<String>, function: Option<String>) -> Result<Expr, ParseError> {
        // We've already consumed the function name, now parse arguments
        self.expect(TokenKind::LeftParen)?;
        
        let mut args = Vec::new();
        if !self.peek().map(|t| t.kind == TokenKind::RightParen).unwrap_or(false) {
            loop {
                args.push(self.parse_expr()?);
                if self.peek().map(|t| t.kind == TokenKind::Comma).unwrap_or(false) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        self.expect(TokenKind::RightParen)?;
        
        let function = function.ok_or_else(|| ParseError::InvalidExpression("Function name missing".to_string()))?;
        
        if module.is_some() {
            Ok(Expr::Call {
                module,
                function,
                args,
            })
        } else {
            Ok(Expr::LocalCall {
                function,
                args,
            })
        }
    }
    
    fn parse_list(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::LeftBracket)?;
        
        if self.peek().map(|t| t.kind == TokenKind::RightBracket).unwrap_or(false) {
            self.advance();
            return Ok(Expr::Nil);
        }
        
        let first = self.parse_expr()?;
        
        match self.peek() {
            Some(tok) if tok.kind == TokenKind::Pipe => {
                // [Head | Tail]
                self.advance();
                let tail = self.parse_expr()?;
                self.expect(TokenKind::RightBracket)?;
                Ok(Expr::Cons {
                    head: Box::new(first),
                    tail: Box::new(tail),
                })
            }
            Some(tok) if tok.kind == TokenKind::Comma => {
                // [E1, E2, ...]
                let mut elems = vec![first];
                loop {
                    self.advance(); // Skip comma
                    elems.push(self.parse_expr()?);
                    if self.peek().map(|t| t.kind == TokenKind::Comma).unwrap_or(false) {
                        continue;
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::RightBracket)?;
                Ok(Expr::List(elems))
            }
            Some(tok) if tok.kind == TokenKind::RightBracket => {
                // [E]
                self.advance();
                Ok(Expr::List(vec![first]))
            }
            _ => Err(ParseError::UnexpectedEof),
        }
    }
    
    fn parse_tuple(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        
        if self.peek().map(|t| t.kind == TokenKind::RightBrace).unwrap_or(false) {
            self.advance();
            return Ok(Expr::Tuple(Vec::new()));
        }
        
        let mut elems = Vec::new();
        elems.push(self.parse_expr()?);
        
        while self.peek().map(|t| t.kind == TokenKind::Comma).unwrap_or(false) {
            self.advance();
            elems.push(self.parse_expr()?);
        }
        
        self.expect(TokenKind::RightBrace)?;
        Ok(Expr::Tuple(elems))
    }
}

/// Parse expressions from tokens
///
/// This is the main entry point for the parser. It parses a list of tokens
/// into a list of expressions.
///
/// # Arguments
/// * `tokens` - List of tokens from erl_scan
///
/// # Returns
/// * `Ok(Vec<Expr>)` - List of parsed expressions
/// * `Err(ParseError)` - Parse error
pub fn parse_exprs(tokens: Vec<Token>) -> Result<Vec<Expr>, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_exprs()
}

/// Parse a single expression from tokens
///
/// # Arguments
/// * `tokens` - List of tokens from erl_scan
///
/// # Returns
/// * `Ok(Expr)` - Parsed expression
/// * `Err(ParseError)` - Parse error
pub fn parse_expr(tokens: Vec<Token>) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_expr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erl_scan::scan_string;
    
    #[test]
    fn test_parse_integer() {
        let tokens = scan_string("123").unwrap();
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(expr, Expr::Integer(123));
    }
    
    #[test]
    fn test_parse_add() {
        let tokens = scan_string("2 + 2").unwrap();
        let expr = parse_expr(tokens).unwrap();
        match expr {
            Expr::BinOp { op: BinOp::Add, left, right } => {
                assert_eq!(*left, Expr::Integer(2));
                assert_eq!(*right, Expr::Integer(2));
            }
            _ => panic!("Expected BinOp::Add"),
        }
    }
    
    #[test]
    fn test_parse_mul() {
        let tokens = scan_string("2 * 3").unwrap();
        let expr = parse_expr(tokens).unwrap();
        match expr {
            Expr::BinOp { op: BinOp::Mul, left, right } => {
                assert_eq!(*left, Expr::Integer(2));
                assert_eq!(*right, Expr::Integer(3));
            }
            _ => panic!("Expected BinOp::Mul"),
        }
    }
    
    #[test]
    fn test_parse_precedence() {
        let tokens = scan_string("2 + 3 * 4").unwrap();
        let expr = parse_expr(tokens).unwrap();
        // Should be: 2 + (3 * 4)
        match expr {
            Expr::BinOp { op: BinOp::Add, left, right } => {
                assert_eq!(*left, Expr::Integer(2));
                match *right {
                    Expr::BinOp { op: BinOp::Mul, left, right } => {
                        assert_eq!(*left, Expr::Integer(3));
                        assert_eq!(*right, Expr::Integer(4));
                    }
                    _ => panic!("Expected nested BinOp::Mul"),
                }
            }
            _ => panic!("Expected BinOp::Add"),
        }
    }
}

