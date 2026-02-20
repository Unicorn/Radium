//! Expression parser
//!
//! Implements a recursive descent parser for simple expressions
//! used in computed variables and conditional logic.

use serde::{Deserialize, Serialize};

/// Expression AST node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Expression {
    // Literals
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Null,

    // Variable references
    Variable(String),

    // Binary arithmetic operations
    Add {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Subtract {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Multiply {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Divide {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Modulo {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // Comparison operations
    Equal {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    NotEqual {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    LessThan {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    GreaterThan {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    LessOrEqual {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    GreaterOrEqual {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // Logical operations
    And {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Or {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Not {
        operand: Box<Expression>,
    },

    // Ternary conditional
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },

    // String operations
    Concat {
        parts: Vec<Expression>,
    },

    // Array operations
    ArrayLength {
        array: Box<Expression>,
    },
    ArrayIncludes {
        array: Box<Expression>,
        item: Box<Expression>,
    },

    // Property access
    Property {
        object: Box<Expression>,
        property: String,
    },
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },

    // Function calls (limited set of safe functions)
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}

impl Expression {
    /// Get all variable names referenced in this expression
    pub fn referenced_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            Expression::Variable(name) => vars.push(name.clone()),
            Expression::Add { left, right }
            | Expression::Subtract { left, right }
            | Expression::Multiply { left, right }
            | Expression::Divide { left, right }
            | Expression::Modulo { left, right }
            | Expression::Equal { left, right }
            | Expression::NotEqual { left, right }
            | Expression::LessThan { left, right }
            | Expression::GreaterThan { left, right }
            | Expression::LessOrEqual { left, right }
            | Expression::GreaterOrEqual { left, right }
            | Expression::And { left, right }
            | Expression::Or { left, right }
            | Expression::ArrayIncludes {
                array: left,
                item: right,
            }
            | Expression::Index {
                array: left,
                index: right,
            } => {
                left.collect_variables(vars);
                right.collect_variables(vars);
            }
            Expression::Not { operand }
            | Expression::ArrayLength { array: operand }
            | Expression::Property {
                object: operand, ..
            } => {
                operand.collect_variables(vars);
            }
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.collect_variables(vars);
                then_branch.collect_variables(vars);
                else_branch.collect_variables(vars);
            }
            Expression::Concat { parts } => {
                for p in parts {
                    p.collect_variables(vars);
                }
            }
            Expression::FunctionCall { args, .. } => {
                for a in args {
                    a.collect_variables(vars);
                }
            }
            Expression::String(_)
            | Expression::Number(_)
            | Expression::Integer(_)
            | Expression::Boolean(_)
            | Expression::Null => {}
        }
    }

    /// Check if this expression is a simple literal
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::String(_)
                | Expression::Number(_)
                | Expression::Integer(_)
                | Expression::Boolean(_)
                | Expression::Null
        )
    }

    /// Check if this expression is a simple variable reference
    pub fn is_variable(&self) -> bool {
        matches!(self, Expression::Variable(_))
    }
}

/// Token types for lexing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Literals
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Null,

    // Identifiers and keywords
    Identifier(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqualEqual,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Not,
    Question,
    Colon,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Dot,
    Comma,

    // End of input
    Eof,
}

/// Expression parser
pub struct ExpressionParser {
    tokens: Vec<Token>,
    current: usize,
}

impl ExpressionParser {
    /// Parse an expression string
    pub fn parse(input: &str) -> Result<Expression, ParseError> {
        let tokens = Self::tokenize(input)?;
        let mut parser = Self { tokens, current: 0 };
        let expr = parser.parse_expression()?;

        // Ensure we consumed all tokens
        if !parser.is_at_end() {
            return Err(ParseError::UnexpectedToken(format!(
                "{:?}",
                parser.peek()
            )));
        }

        Ok(expr)
    }

    fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    chars.next();
                }
                '+' => {
                    chars.next();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    chars.next();
                    // Check for negative number
                    if let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() && tokens.last().map_or(true, |t| {
                            matches!(t, Token::Plus | Token::Minus | Token::Star | Token::Slash |
                                     Token::Percent | Token::LeftParen | Token::LeftBracket |
                                     Token::Comma | Token::Question | Token::Colon |
                                     Token::EqualEqual | Token::NotEqual | Token::Less |
                                     Token::Greater | Token::LessEqual | Token::GreaterEqual |
                                     Token::And | Token::Or | Token::Not)
                        }) {
                            // This is a negative number
                            let mut num = String::from("-");
                            while let Some(&ch) = chars.peek() {
                                if ch.is_ascii_digit() || ch == '.' {
                                    num.push(ch);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if num.contains('.') {
                                let n: f64 = num
                                    .parse()
                                    .map_err(|_| ParseError::InvalidNumber(num.clone()))?;
                                tokens.push(Token::Number(n));
                            } else {
                                let n: i64 = num
                                    .parse()
                                    .map_err(|_| ParseError::InvalidNumber(num.clone()))?;
                                tokens.push(Token::Integer(n));
                            }
                            continue;
                        }
                    }
                    tokens.push(Token::Minus);
                }
                '*' => {
                    chars.next();
                    tokens.push(Token::Star);
                }
                '/' => {
                    chars.next();
                    tokens.push(Token::Slash);
                }
                '%' => {
                    chars.next();
                    tokens.push(Token::Percent);
                }
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::EqualEqual);
                    } else {
                        return Err(ParseError::UnexpectedToken("=".to_string()));
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::NotEqual);
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                '&' => {
                    chars.next();
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        tokens.push(Token::And);
                    } else {
                        return Err(ParseError::UnexpectedToken("&".to_string()));
                    }
                }
                '|' => {
                    chars.next();
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        tokens.push(Token::Or);
                    } else {
                        return Err(ParseError::UnexpectedToken("|".to_string()));
                    }
                }
                '?' => {
                    chars.next();
                    tokens.push(Token::Question);
                }
                ':' => {
                    chars.next();
                    tokens.push(Token::Colon);
                }
                '(' => {
                    chars.next();
                    tokens.push(Token::LeftParen);
                }
                ')' => {
                    chars.next();
                    tokens.push(Token::RightParen);
                }
                '[' => {
                    chars.next();
                    tokens.push(Token::LeftBracket);
                }
                ']' => {
                    chars.next();
                    tokens.push(Token::RightBracket);
                }
                '.' => {
                    chars.next();
                    tokens.push(Token::Dot);
                }
                ',' => {
                    chars.next();
                    tokens.push(Token::Comma);
                }
                '"' | '\'' => {
                    let quote = c;
                    chars.next();
                    let mut s = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == quote {
                            chars.next();
                            break;
                        } else if ch == '\\' {
                            chars.next();
                            if let Some(&escaped) = chars.peek() {
                                chars.next();
                                match escaped {
                                    'n' => s.push('\n'),
                                    't' => s.push('\t'),
                                    'r' => s.push('\r'),
                                    '\\' => s.push('\\'),
                                    '"' => s.push('"'),
                                    '\'' => s.push('\''),
                                    _ => s.push(escaped),
                                }
                            }
                        } else {
                            s.push(ch);
                            chars.next();
                        }
                    }
                    tokens.push(Token::String(s));
                }
                _ if c.is_ascii_digit() => {
                    let mut num = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() || ch == '.' {
                            num.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if num.contains('.') {
                        let n: f64 = num
                            .parse()
                            .map_err(|_| ParseError::InvalidNumber(num.clone()))?;
                        tokens.push(Token::Number(n));
                    } else {
                        let n: i64 = num
                            .parse()
                            .map_err(|_| ParseError::InvalidNumber(num.clone()))?;
                        tokens.push(Token::Integer(n));
                    }
                }
                _ if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            ident.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    match ident.as_str() {
                        "true" => tokens.push(Token::Boolean(true)),
                        "false" => tokens.push(Token::Boolean(false)),
                        "null" => tokens.push(Token::Null),
                        _ => tokens.push(Token::Identifier(ident)),
                    }
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(c.to_string()));
                }
            }
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_or()?;

        if self.check(&Token::Question) {
            self.advance();
            let then_branch = self.parse_expression()?;
            self.consume(&Token::Colon, "Expected ':' in ternary expression")?;
            let else_branch = self.parse_expression()?;
            expr = Expression::Conditional {
                condition: Box::new(expr),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            };
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_and()?;

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::Or {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::And {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        loop {
            if self.check(&Token::EqualEqual) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expression::Equal {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&Token::NotEqual) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expression::NotEqual {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_term()?;

        loop {
            if self.check(&Token::Less) {
                self.advance();
                let right = self.parse_term()?;
                left = Expression::LessThan {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&Token::Greater) {
                self.advance();
                let right = self.parse_term()?;
                left = Expression::GreaterThan {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&Token::LessEqual) {
                self.advance();
                let right = self.parse_term()?;
                left = Expression::LessOrEqual {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&Token::GreaterEqual) {
                self.advance();
                let right = self.parse_term()?;
                left = Expression::GreaterOrEqual {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_factor()?;

        loop {
            if self.check(&Token::Plus) {
                self.advance();
                let right = self.parse_factor()?;
                left = Expression::Add {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&Token::Minus) {
                self.advance();
                let right = self.parse_factor()?;
                left = Expression::Subtract {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            if self.check(&Token::Star) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::Multiply {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&Token::Slash) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::Divide {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.check(&Token::Percent) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::Modulo {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.check(&Token::Not) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::Not {
                operand: Box::new(operand),
            });
        }
        if self.check(&Token::Minus) {
            self.advance();
            let operand = self.parse_unary()?;
            // Convert to subtraction from 0
            return Ok(Expression::Subtract {
                left: Box::new(Expression::Integer(0)),
                right: Box::new(operand),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&Token::Dot) {
                self.advance();
                if let Some(Token::Identifier(name)) = self.peek().cloned() {
                    self.advance();
                    // Check for method call (e.g., .length, .includes())
                    if self.check(&Token::LeftParen) {
                        self.advance();
                        let mut args = vec![expr.clone()];
                        if !self.check(&Token::RightParen) {
                            loop {
                                args.push(self.parse_expression()?);
                                if !self.check(&Token::Comma) {
                                    break;
                                }
                                self.advance();
                            }
                        }
                        self.consume(&Token::RightParen, "Expected ')' after arguments")?;

                        expr = match name.as_str() {
                            "length" => Expression::ArrayLength {
                                array: Box::new(args.remove(0)),
                            },
                            "includes" if args.len() == 2 => Expression::ArrayIncludes {
                                array: Box::new(args.remove(0)),
                                item: Box::new(args.remove(0)),
                            },
                            _ => Expression::FunctionCall {
                                name: format!("{}.{}", self.expr_to_string(&expr), name),
                                args: args[1..].to_vec(),
                            },
                        };
                    } else {
                        expr = Expression::Property {
                            object: Box::new(expr),
                            property: name,
                        };
                    }
                } else {
                    return Err(ParseError::UnexpectedEnd);
                }
            } else if self.check(&Token::LeftBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.consume(&Token::RightBracket, "Expected ']' after index")?;
                expr = Expression::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.peek().cloned();

        match token {
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expression::String(s))
            }
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expression::Number(n))
            }
            Some(Token::Integer(n)) => {
                self.advance();
                Ok(Expression::Integer(n))
            }
            Some(Token::Boolean(b)) => {
                self.advance();
                Ok(Expression::Boolean(b))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expression::Null)
            }
            Some(Token::Identifier(name)) => {
                self.advance();
                // Check for function call
                if self.check(&Token::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.check(&Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    self.consume(&Token::RightParen, "Expected ')' after arguments")?;
                    Ok(Expression::FunctionCall { name, args })
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Some(Token::LeftBracket) => {
                // Array literal
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.consume(&Token::RightBracket, "Expected ']' after array")?;
                // Represent array as a function call for simplicity
                Ok(Expression::FunctionCall {
                    name: "Array".to_string(),
                    args: elements,
                })
            }
            Some(Token::Eof) => Err(ParseError::UnexpectedEnd),
            Some(t) => Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn expr_to_string(&self, _expr: &Expression) -> String {
        // Simple helper for building method call names
        "expr".to_string()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn check(&self, token: &Token) -> bool {
        self.peek().map(|t| std::mem::discriminant(t) == std::mem::discriminant(token)).unwrap_or(false)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::Eof) | None)
    }

    fn consume(&mut self, token: &Token, message: &str) -> Result<(), ParseError> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(message.to_string()))
        }
    }
}

/// Parse error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),

    #[error("Unexpected end of expression")]
    UnexpectedEnd,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unknown function: {0}")]
    UnknownFunction(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literals() {
        assert_eq!(
            ExpressionParser::parse("42").unwrap(),
            Expression::Integer(42)
        );
        assert_eq!(
            ExpressionParser::parse("3.14").unwrap(),
            Expression::Number(3.14)
        );
        assert_eq!(
            ExpressionParser::parse("\"hello\"").unwrap(),
            Expression::String("hello".to_string())
        );
        assert_eq!(
            ExpressionParser::parse("'world'").unwrap(),
            Expression::String("world".to_string())
        );
        assert_eq!(
            ExpressionParser::parse("true").unwrap(),
            Expression::Boolean(true)
        );
        assert_eq!(
            ExpressionParser::parse("false").unwrap(),
            Expression::Boolean(false)
        );
        assert_eq!(ExpressionParser::parse("null").unwrap(), Expression::Null);
    }

    #[test]
    fn test_parse_variables() {
        assert_eq!(
            ExpressionParser::parse("foo").unwrap(),
            Expression::Variable("foo".to_string())
        );
        assert_eq!(
            ExpressionParser::parse("my_var").unwrap(),
            Expression::Variable("my_var".to_string())
        );
    }

    #[test]
    fn test_parse_arithmetic() {
        let expr = ExpressionParser::parse("1 + 2").unwrap();
        assert!(matches!(expr, Expression::Add { .. }));

        let expr = ExpressionParser::parse("3 - 1").unwrap();
        assert!(matches!(expr, Expression::Subtract { .. }));

        let expr = ExpressionParser::parse("2 * 3").unwrap();
        assert!(matches!(expr, Expression::Multiply { .. }));

        let expr = ExpressionParser::parse("6 / 2").unwrap();
        assert!(matches!(expr, Expression::Divide { .. }));

        let expr = ExpressionParser::parse("7 % 3").unwrap();
        assert!(matches!(expr, Expression::Modulo { .. }));
    }

    #[test]
    fn test_parse_comparison() {
        let expr = ExpressionParser::parse("a == b").unwrap();
        assert!(matches!(expr, Expression::Equal { .. }));

        let expr = ExpressionParser::parse("a != b").unwrap();
        assert!(matches!(expr, Expression::NotEqual { .. }));

        let expr = ExpressionParser::parse("a < b").unwrap();
        assert!(matches!(expr, Expression::LessThan { .. }));

        let expr = ExpressionParser::parse("a > b").unwrap();
        assert!(matches!(expr, Expression::GreaterThan { .. }));

        let expr = ExpressionParser::parse("a <= b").unwrap();
        assert!(matches!(expr, Expression::LessOrEqual { .. }));

        let expr = ExpressionParser::parse("a >= b").unwrap();
        assert!(matches!(expr, Expression::GreaterOrEqual { .. }));
    }

    #[test]
    fn test_parse_logical() {
        let expr = ExpressionParser::parse("a && b").unwrap();
        assert!(matches!(expr, Expression::And { .. }));

        let expr = ExpressionParser::parse("a || b").unwrap();
        assert!(matches!(expr, Expression::Or { .. }));

        let expr = ExpressionParser::parse("!a").unwrap();
        assert!(matches!(expr, Expression::Not { .. }));
    }

    #[test]
    fn test_parse_ternary() {
        let expr = ExpressionParser::parse("a ? b : c").unwrap();
        assert!(matches!(expr, Expression::Conditional { .. }));
    }

    #[test]
    fn test_parse_property_access() {
        let expr = ExpressionParser::parse("obj.prop").unwrap();
        assert!(matches!(expr, Expression::Property { .. }));

        let expr = ExpressionParser::parse("arr[0]").unwrap();
        assert!(matches!(expr, Expression::Index { .. }));
    }

    #[test]
    fn test_parse_function_call() {
        let expr = ExpressionParser::parse("foo(1, 2)").unwrap();
        if let Expression::FunctionCall { name, args } = expr {
            assert_eq!(name, "foo");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected FunctionCall");
        }
    }

    #[test]
    fn test_parse_method_call_length() {
        let expr = ExpressionParser::parse("arr.length()").unwrap();
        assert!(matches!(expr, Expression::ArrayLength { .. }));
    }

    #[test]
    fn test_parse_method_call_includes() {
        let expr = ExpressionParser::parse("arr.includes(x)").unwrap();
        assert!(matches!(expr, Expression::ArrayIncludes { .. }));
    }

    #[test]
    fn test_operator_precedence() {
        // Multiplication before addition
        let expr = ExpressionParser::parse("1 + 2 * 3").unwrap();
        if let Expression::Add { right, .. } = expr {
            assert!(matches!(*right, Expression::Multiply { .. }));
        } else {
            panic!("Expected Add at top level");
        }

        // Parentheses override
        let expr = ExpressionParser::parse("(1 + 2) * 3").unwrap();
        if let Expression::Multiply { left, .. } = expr {
            assert!(matches!(*left, Expression::Add { .. }));
        } else {
            panic!("Expected Multiply at top level");
        }
    }

    #[test]
    fn test_referenced_variables() {
        let expr = ExpressionParser::parse("a + b * c").unwrap();
        let vars = expr.referenced_variables();
        assert_eq!(vars, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_complex_expression() {
        let expr = ExpressionParser::parse("x > 0 && y < 10 ? x + y : x - y").unwrap();
        assert!(matches!(expr, Expression::Conditional { .. }));
    }

    #[test]
    fn test_negative_numbers() {
        // At the start of an expression, -5 is parsed as a negative literal
        let expr = ExpressionParser::parse("-5").unwrap();
        assert_eq!(expr, Expression::Integer(-5));

        // In parentheses, same behavior
        let expr = ExpressionParser::parse("(-5)").unwrap();
        assert_eq!(expr, Expression::Integer(-5));

        // In an arithmetic context, - is subtraction
        let expr = ExpressionParser::parse("10 - 5").unwrap();
        assert!(matches!(expr, Expression::Subtract { .. }));
    }

    #[test]
    fn test_string_escapes() {
        let expr = ExpressionParser::parse(r#""hello\nworld""#).unwrap();
        if let Expression::String(s) = expr {
            assert_eq!(s, "hello\nworld");
        } else {
            panic!("Expected String");
        }
    }
}
