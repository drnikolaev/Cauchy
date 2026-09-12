use super::MathExpr;
use num_traits::Float;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "parse error at byte {}: {}",
            self.position, self.message
        )
    }
}

impl Error for ParseError {}

impl<'a, T> MathExpr<'a, T>
where
    T: Float + Debug + FromStr,
{
    pub fn parse(input: &'a str) -> Result<Self, ParseError> {
        let mut parser = Parser::<T>::new(input);
        let expression = parser.parse_expression()?;
        parser.skip_whitespace();

        if parser.is_at_end() {
            Ok(expression)
        } else {
            Err(parser.error("unexpected trailing input"))
        }
    }
}

struct Parser<'a, T> {
    input: &'a str,
    position: usize,
    marker: PhantomData<T>,
}

impl<'a, T> Parser<'a, T>
where
    T: Float + Debug + FromStr,
{
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            marker: PhantomData,
        }
    }

    fn parse_expression(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        let mut expression = self.parse_multiplication()?;

        loop {
            if self.consume_symbol('+') {
                expression = MathExpr::new_add(expression, self.parse_multiplication()?);
            } else if self.consume_symbol('-') {
                expression = MathExpr::new_subtract(expression, self.parse_multiplication()?);
            } else {
                return Ok(expression);
            }
        }
    }

    fn parse_multiplication(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        let mut expression = self.parse_unary()?;

        loop {
            if self.consume_symbol('*') {
                expression = MathExpr::new_multiply(expression, self.parse_unary()?);
            } else if self.consume_symbol('/') {
                expression = MathExpr::new_divide(expression, self.parse_unary()?);
            } else {
                return Ok(expression);
            }
        }
    }

    fn parse_unary(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        if self.consume_symbol('-') {
            Ok(MathExpr::new_negate(self.parse_unary()?))
        } else {
            self.parse_power()
        }
    }

    fn parse_power(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        let base = self.parse_primary()?;

        if self.consume_symbol('^') {
            let exponent = self.parse_unary()?;
            Ok(MathExpr::new_power(base, exponent))
        } else {
            Ok(base)
        }
    }

    fn parse_primary(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        self.skip_whitespace();

        if self.consume_symbol('(') {
            let expression = self.parse_expression()?;
            self.expect_symbol(')')?;
            return Ok(expression);
        }

        match self.peek_char() {
            Some(character) if character.is_ascii_digit() || character == '.' => {
                self.parse_number()
            }
            Some(character) if character.is_alphabetic() || character == '_' => self.parse_name(),
            Some(character) => Err(self.error(format!("unexpected character '{character}'"))),
            None => Err(self.error("expected an expression")),
        }
    }

    fn parse_number(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        self.parse_number_value().map(MathExpr::new_const)
    }

    fn parse_number_value(&mut self) -> Result<T, ParseError> {
        self.skip_whitespace();
        let start = self.position;
        self.consume_digits();

        if self.peek_char() == Some('.') {
            self.advance_char();
            self.consume_digits();
        }

        if matches!(self.peek_char(), Some('e' | 'E')) {
            self.advance_char();
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.advance_char();
            }

            let exponent_start = self.position;
            self.consume_digits();
            if exponent_start == self.position {
                return Err(self.error("expected digits after the exponent"));
            }
        }

        let token = &self.input[start..self.position];
        token.parse::<T>().map_err(|_| ParseError {
            position: start,
            message: format!("invalid number '{token}'"),
        })
    }

    fn parse_name(&mut self) -> Result<MathExpr<'a, T>, ParseError> {
        let start = self.position;
        self.advance_char();

        while matches!(self.peek_char(), Some(character) if character.is_alphanumeric() || character == '_')
        {
            self.advance_char();
        }

        let name = &self.input[start..self.position];
        self.skip_whitespace();

        if self.consume_symbol('(') {
            if name == "iif" {
                let check = self.parse_expression()?;
                self.expect_symbol(',')?;
                let if_less_than_zero = self.parse_expression()?;
                self.expect_symbol(',')?;
                let otherwise = self.parse_expression()?;
                self.expect_symbol(')')?;

                return Ok(MathExpr::new_iif(check, if_less_than_zero, otherwise));
            }

            let operand = self.parse_expression()?;
            self.expect_symbol(')')?;

            return match name {
                "sin" => Ok(MathExpr::new_sin(operand)),
                "cos" => Ok(MathExpr::new_cos(operand)),
                "tan" => Ok(MathExpr::new_tan(operand)),
                "asin" => Ok(MathExpr::new_asin(operand)),
                "acos" => Ok(MathExpr::new_acos(operand)),
                "atan" => Ok(MathExpr::new_atan(operand)),
                "sinh" => Ok(MathExpr::new_sinh(operand)),
                "cosh" => Ok(MathExpr::new_cosh(operand)),
                "tanh" => Ok(MathExpr::new_tanh(operand)),
                "sign" => Ok(MathExpr::new_sign(operand)),
                "abs" => Ok(MathExpr::new_abs(operand)),
                "exp" => Ok(MathExpr::new_exp(operand)),
                "sqrt" => Ok(MathExpr::new_sqrt(operand)),
                "log" => Ok(MathExpr::new_log(operand)),
                "log10" => Ok(MathExpr::new_log10(operand)),
                _ => Err(ParseError {
                    position: start,
                    message: format!("unknown function '{name}'"),
                }),
            };
        }

        if let Ok(value) = name.parse::<T>() {
            Ok(MathExpr::new_const(value))
        } else {
            Ok(MathExpr::new_var(name))
        }
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek_char(), Some(character) if character.is_ascii_digit()) {
            self.advance_char();
        }
    }

    fn consume_symbol(&mut self, expected: char) -> bool {
        self.skip_whitespace();
        if self.peek_char() == Some(expected) {
            self.advance_char();
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, expected: char) -> Result<(), ParseError> {
        if self.consume_symbol(expected) {
            Ok(())
        } else {
            Err(self.error(format!("expected '{expected}'")))
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(character) if character.is_whitespace()) {
            self.advance_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance_char(&mut self) {
        if let Some(character) = self.peek_char() {
            self.position += character.len_utf8();
        }
    }

    fn is_at_end(&self) -> bool {
        self.position == self.input.len()
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            position: self.position,
            message: message.into(),
        }
    }
}
