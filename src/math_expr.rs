use num_traits::Float;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

mod parser;
pub use parser::ParseError;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation<'a, T> {
    Variable(&'a str),
    Constant(T),
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Negate,
    Sine,
    Cosine,
    Tangent,
    ArcSine,
    ArcCosine,
    ArcTangent,
    HyperbolicSine,
    HyperbolicCosine,
    HyperbolicTangent,
    Sign,
    AbsoluteValue,
    Exponent,
    SquareRoot,
    NaturalLogarithm,
    CommonLogarithm,
    ImmediateIf,
}

#[derive(Debug, PartialEq)]
pub enum EvaluationError<'a, T> {
    InvalidOperandCount {
        operation: Operation<'a, T>,
        expected: usize,
        actual: usize,
    },
    VariableNotDefined(&'a str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct MathExpr<'a, T: Float> {
    operation: Operation<'a, T>,
    operands: Vec<MathExpr<'a, T>>,
}

impl<'a, T: Float + std::fmt::Debug> MathExpr<'a, T> {
    fn new(operation: Operation<'a, T>, operands: Vec<MathExpr<'a, T>>) -> Self {
        Self {
            operation,
            operands,
        }
    }

    pub fn new_var(name: &'a str) -> Self {
        Self {
            operation: Operation::Variable(name),
            operands: vec![],
        }
    }
    pub fn new_const(value: T) -> Self {
        Self {
            operation: Operation::Constant(value),
            operands: vec![],
        }
    }

    pub fn new_add(left: Self, right: Self) -> Self {
        Self::new(Operation::Add, vec![left, right])
    }

    pub fn new_subtract(left: Self, right: Self) -> Self {
        Self::new(Operation::Subtract, vec![left, right])
    }

    pub fn new_multiply(left: Self, right: Self) -> Self {
        Self::new(Operation::Multiply, vec![left, right])
    }

    pub fn new_divide(numerator: Self, denominator: Self) -> Self {
        Self::new(Operation::Divide, vec![numerator, denominator])
    }

    pub fn new_power(base: Self, exponent: Self) -> Self {
        let negated_constant = match (&exponent.operation, exponent.operands.as_slice()) {
            (Operation::Negate, [operand]) => operand.constant_value().map(|value| -value),
            _ => None,
        };
        let exponent = match negated_constant {
            Some(value) => Self::new_const(value),
            None => exponent,
        };

        Self::new(Operation::Power, vec![base, exponent])
    }

    pub fn new_negate(operand: Self) -> Self {
        Self::new(Operation::Negate, vec![operand])
    }

    pub fn new_sin(operand: Self) -> Self {
        Self::new(Operation::Sine, vec![operand])
    }

    pub fn new_cos(operand: Self) -> Self {
        Self::new(Operation::Cosine, vec![operand])
    }

    pub fn new_tan(operand: Self) -> Self {
        Self::new(Operation::Tangent, vec![operand])
    }

    pub fn new_asin(operand: Self) -> Self {
        Self::new(Operation::ArcSine, vec![operand])
    }

    pub fn new_acos(operand: Self) -> Self {
        Self::new(Operation::ArcCosine, vec![operand])
    }

    pub fn new_atan(operand: Self) -> Self {
        Self::new(Operation::ArcTangent, vec![operand])
    }

    pub fn new_sinh(operand: Self) -> Self {
        Self::new(Operation::HyperbolicSine, vec![operand])
    }

    pub fn new_cosh(operand: Self) -> Self {
        Self::new(Operation::HyperbolicCosine, vec![operand])
    }

    pub fn new_tanh(operand: Self) -> Self {
        Self::new(Operation::HyperbolicTangent, vec![operand])
    }

    pub fn new_sign(operand: Self) -> Self {
        Self::new(Operation::Sign, vec![operand])
    }

    pub fn new_abs(operand: Self) -> Self {
        Self::new(Operation::AbsoluteValue, vec![operand])
    }

    pub fn new_exp(operand: Self) -> Self {
        Self::new(Operation::Exponent, vec![operand])
    }

    pub fn new_sqrt(operand: Self) -> Self {
        Self::new(Operation::SquareRoot, vec![operand])
    }

    pub fn new_ln(operand: Self) -> Self {
        Self::new(Operation::NaturalLogarithm, vec![operand])
    }

    pub fn new_log10(operand: Self) -> Self {
        Self::new(Operation::CommonLogarithm, vec![operand])
    }

    pub fn new_iif(check: Self, if_less_than_zero: Self, otherwise: Self) -> Self {
        Self::new(
            Operation::ImmediateIf,
            vec![check, if_less_than_zero, otherwise],
        )
    }

    pub fn derive(&self, by_var: &str) -> Self {
        let two = T::from(2).expect("2 must be representable by floating-point types");

        match &self.operation {
            Operation::Constant(_) => Self::new_const(T::zero()),
            Operation::Variable(name) => {
                if *name == by_var {
                    Self::new_const(T::one())
                } else {
                    Self::new_const(T::zero())
                }
            }
            Operation::Add => {
                let (left, right) = self.binary_operands(Operation::Add).unwrap();
                Self::new_add(left.derive(by_var), right.derive(by_var))
            }
            Operation::Subtract => {
                let (left, right) = self.binary_operands(Operation::Subtract).unwrap();
                Self::new_subtract(left.derive(by_var), right.derive(by_var))
            }
            Operation::Multiply => {
                let (left, right) = self.binary_operands(Operation::Multiply).unwrap();
                Self::new_add(
                    Self::new_multiply(left.derive(by_var), (*right).clone()),
                    Self::new_multiply((*left).clone(), right.derive(by_var)),
                )
            }
            Operation::Divide => {
                let (numerator, denominator) = self.binary_operands(Operation::Divide).unwrap();
                Self::new_divide(
                    Self::new_subtract(
                        Self::new_multiply(numerator.derive(by_var), (*denominator).clone()),
                        Self::new_multiply((*numerator).clone(), denominator.derive(by_var)),
                    ),
                    Self::new_multiply((*denominator).clone(), (*denominator).clone()),
                )
            }
            Operation::Power => {
                let (base, exponent) = self.binary_operands(Operation::Power).unwrap();
                match exponent.constant_value() {
                    Some(value) if value == T::zero() => Self::new_const(T::zero()),
                    Some(value) if value == T::one() => base.derive(by_var),
                    Some(value) => Self::new_multiply(
                        Self::new_multiply(
                            Self::new_const(value),
                            Self::new_power(base.clone(), Self::new_const(value - T::one())),
                        ),
                        base.derive(by_var),
                    ),
                    None => Self::new_multiply(
                        self.clone(),
                        Self::new_add(
                            Self::new_multiply(exponent.derive(by_var), Self::new_ln(base.clone())),
                            Self::new_multiply(
                                exponent.clone(),
                                Self::new_divide(base.derive(by_var), base.clone()),
                            ),
                        ),
                    ),
                }
            }
            Operation::Negate => {
                let (operand,) = self.unary_operands(Operation::Negate).unwrap();
                Self::new_negate(operand.derive(by_var))
            }
            Operation::Sine => {
                let (operand,) = self.unary_operands(Operation::Sine).unwrap();
                Self::new_multiply(operand.derive(by_var), Self::new_cos(operand.clone()))
            }
            Operation::Cosine => {
                let (operand,) = self.unary_operands(Operation::Cosine).unwrap();
                Self::new_multiply(
                    operand.derive(by_var),
                    Self::new_negate(Self::new_sin(operand.clone())),
                )
            }
            Operation::Tangent => {
                let (operand,) = self.unary_operands(Operation::Tangent).unwrap();
                Self::new_divide(
                    operand.derive(by_var),
                    Self::new_power(Self::new_cos(operand.clone()), Self::new_const(two)),
                )
            }
            Operation::ArcSine => {
                let (operand,) = self.unary_operands(Operation::ArcSine).unwrap();
                Self::new_divide(
                    operand.derive(by_var),
                    Self::new_sqrt(Self::new_subtract(
                        Self::new_const(T::one()),
                        Self::new_power(operand.clone(), Self::new_const(two)),
                    )),
                )
            }
            Operation::ArcCosine => {
                let (operand,) = self.unary_operands(Operation::ArcCosine).unwrap();
                Self::new_divide(
                    Self::new_negate(operand.derive(by_var)),
                    Self::new_sqrt(Self::new_subtract(
                        Self::new_const(T::one()),
                        Self::new_power(operand.clone(), Self::new_const(two)),
                    )),
                )
            }
            Operation::ArcTangent => {
                let (operand,) = self.unary_operands(Operation::ArcTangent).unwrap();
                Self::new_divide(
                    operand.derive(by_var),
                    Self::new_add(
                        Self::new_const(T::one()),
                        Self::new_power(operand.clone(), Self::new_const(two)),
                    ),
                )
            }
            Operation::HyperbolicSine => {
                let (operand,) = self.unary_operands(Operation::HyperbolicSine).unwrap();
                Self::new_multiply(operand.derive(by_var), Self::new_cosh(operand.clone()))
            }
            Operation::HyperbolicCosine => {
                let (operand,) = self.unary_operands(Operation::HyperbolicCosine).unwrap();
                Self::new_multiply(operand.derive(by_var), Self::new_sinh(operand.clone()))
            }
            Operation::HyperbolicTangent => {
                let (operand,) = self.unary_operands(Operation::HyperbolicTangent).unwrap();
                Self::new_divide(
                    operand.derive(by_var),
                    Self::new_power(Self::new_cosh(operand.clone()), Self::new_const(two)),
                )
            }
            Operation::Sign => Self::new_const(T::zero()),
            Operation::AbsoluteValue => {
                let (operand,) = self.unary_operands(Operation::AbsoluteValue).unwrap();
                Self::new_multiply(operand.derive(by_var), Self::new_sign(operand.clone()))
            }
            Operation::Exponent => {
                let (operand,) = self.unary_operands(Operation::Exponent).unwrap();
                Self::new_multiply(operand.derive(by_var), Self::new_exp(operand.clone()))
            }
            Operation::SquareRoot => {
                let (operand,) = self.unary_operands(Operation::SquareRoot).unwrap();
                Self::new_divide(
                    operand.derive(by_var),
                    Self::new_multiply(Self::new_const(two), Self::new_sqrt(operand.clone())),
                )
            }
            Operation::NaturalLogarithm => {
                let (operand,) = self.unary_operands(Operation::NaturalLogarithm).unwrap();
                Self::new_divide(operand.derive(by_var), operand.clone())
            }
            Operation::CommonLogarithm => {
                let (operand,) = self.unary_operands(Operation::CommonLogarithm).unwrap();
                let ten = T::from(10).expect("10 must be representable by floating-point types");
                Self::new_divide(
                    operand.derive(by_var),
                    Self::new_multiply(operand.clone(), Self::new_const(ten.ln())),
                )
            }
            Operation::ImmediateIf => {
                let (check, if_less_than_zero, otherwise) =
                    self.ternary_operands(Operation::ImmediateIf).unwrap();
                Self::new_iif(
                    check.clone(),
                    if_less_than_zero.derive(by_var),
                    otherwise.derive(by_var),
                )
            }
        }
    }

    pub fn simplify(&self) -> Self {
        match &self.operation {
            Operation::Variable(name) => Self::new_var(name),
            Operation::Constant(value) => Self::new_const(*value),
            Operation::Add => {
                let (left, right) = self.binary_operands(Operation::Add).unwrap();
                Self::simplify_sum(left.simplify(), right.simplify(), false)
            }
            Operation::Subtract => {
                let (left, right) = self.binary_operands(Operation::Subtract).unwrap();
                Self::simplify_sum(left.simplify(), right.simplify(), true)
            }
            Operation::Multiply => {
                let (left, right) = self.binary_operands(Operation::Multiply).unwrap();
                Self::simplify_product(left.simplify(), right.simplify())
            }
            Operation::Divide => {
                let (numerator, denominator) = self.binary_operands(Operation::Divide).unwrap();
                Self::simplify_quotient(numerator.simplify(), denominator.simplify())
            }
            Operation::Power => {
                let (base, exponent) = self.binary_operands(Operation::Power).unwrap();
                Self::simplify_power(base.simplify(), exponent.simplify())
            }
            Operation::Negate => {
                let (operand,) = self.unary_operands(Operation::Negate).unwrap();
                Self::simplify_negation(operand.simplify())
            }
            Operation::Sine => {
                let (operand,) = self.unary_operands(Operation::Sine).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.sin()),
                    None => Self::new_sin(operand),
                }
            }
            Operation::Cosine => {
                let (operand,) = self.unary_operands(Operation::Cosine).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.cos()),
                    None => Self::new_cos(operand),
                }
            }
            Operation::Tangent => {
                let (operand,) = self.unary_operands(Operation::Tangent).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.tan()),
                    None => Self::new_tan(operand),
                }
            }
            Operation::ArcSine => {
                let (operand,) = self.unary_operands(Operation::ArcSine).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.asin()),
                    None => Self::new_asin(operand),
                }
            }
            Operation::ArcCosine => {
                let (operand,) = self.unary_operands(Operation::ArcCosine).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.acos()),
                    None => Self::new_acos(operand),
                }
            }
            Operation::ArcTangent => {
                let (operand,) = self.unary_operands(Operation::ArcTangent).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.atan()),
                    None => Self::new_atan(operand),
                }
            }
            Operation::HyperbolicSine => {
                let (operand,) = self.unary_operands(Operation::HyperbolicSine).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.sinh()),
                    None => Self::new_sinh(operand),
                }
            }
            Operation::HyperbolicCosine => {
                let (operand,) = self.unary_operands(Operation::HyperbolicCosine).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.cosh()),
                    None => Self::new_cosh(operand),
                }
            }
            Operation::HyperbolicTangent => {
                let (operand,) = self.unary_operands(Operation::HyperbolicTangent).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.tanh()),
                    None => Self::new_tanh(operand),
                }
            }
            Operation::Sign => {
                let (operand,) = self.unary_operands(Operation::Sign).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(Self::sign_value(value)),
                    None => Self::new_sign(operand),
                }
            }
            Operation::AbsoluteValue => {
                let (operand,) = self.unary_operands(Operation::AbsoluteValue).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.abs()),
                    None => Self::new_abs(operand),
                }
            }
            Operation::Exponent => {
                let (operand,) = self.unary_operands(Operation::Exponent).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.exp()),
                    None => Self::new_exp(operand),
                }
            }
            Operation::SquareRoot => {
                let (operand,) = self.unary_operands(Operation::SquareRoot).unwrap();
                let operand = operand.simplify();
                match operand.constant_value() {
                    Some(value) => Self::new_const(value.sqrt()),
                    None => Self::new_sqrt(operand),
                }
            }
            Operation::NaturalLogarithm => {
                let (operand,) = self.unary_operands(Operation::NaturalLogarithm).unwrap();
                Self::simplify_natural_logarithm(operand.simplify())
            }
            Operation::CommonLogarithm => {
                let (operand,) = self.unary_operands(Operation::CommonLogarithm).unwrap();
                Self::simplify_common_logarithm(operand.simplify())
            }
            Operation::ImmediateIf => {
                let (check, if_less_than_zero, otherwise) =
                    self.ternary_operands(Operation::ImmediateIf).unwrap();
                let check = check.simplify();

                match check.constant_value() {
                    Some(value) if value < T::zero() => if_less_than_zero.simplify(),
                    Some(_) => otherwise.simplify(),
                    None => {
                        Self::new_iif(check, if_less_than_zero.simplify(), otherwise.simplify())
                    }
                }
            }
        }
    }

    fn simplify_sum(left: Self, right: Self, subtract_right: bool) -> Self {
        let mut terms = Vec::new();
        let mut constant = T::zero();

        Self::collect_addends(&left, T::one(), &mut terms, &mut constant);
        let right_sign = if subtract_right { -T::one() } else { T::one() };
        Self::collect_addends(&right, right_sign, &mut terms, &mut constant);

        let mut result = None;
        for (coefficient, term) in terms {
            if coefficient == T::zero() {
                continue;
            }

            let term = Self::scale_term(coefficient, term);
            result = Some(match result {
                Some(expression) => Self::new_add(expression, term),
                None => term,
            });
        }

        if constant != T::zero() {
            let constant = Self::new_const(constant);
            result = Some(match result {
                Some(expression) => Self::new_add(expression, constant),
                None => constant,
            });
        }

        result.unwrap_or_else(|| Self::new_const(T::zero()))
    }

    fn collect_addends(expression: &Self, sign: T, terms: &mut Vec<(T, Self)>, constant: &mut T) {
        match (&expression.operation, expression.operands.as_slice()) {
            (Operation::Add, [left, right]) => {
                Self::collect_addends(left, sign, terms, constant);
                Self::collect_addends(right, sign, terms, constant);
            }
            (Operation::Subtract, [left, right]) => {
                Self::collect_addends(left, sign, terms, constant);
                Self::collect_addends(right, -sign, terms, constant);
            }
            (Operation::Constant(value), []) => *constant = *constant + sign * *value,
            _ => {
                let (coefficient, term) = expression.coefficient_and_term();
                let coefficient = sign * coefficient;

                if let Some((existing_coefficient, _)) =
                    terms.iter_mut().find(|(_, existing)| existing == &term)
                {
                    *existing_coefficient = *existing_coefficient + coefficient;
                } else {
                    terms.push((coefficient, term));
                }
            }
        }
    }

    fn coefficient_and_term(&self) -> (T, Self) {
        match (&self.operation, self.operands.as_slice()) {
            (Operation::Multiply, [coefficient, term]) => match coefficient.constant_value() {
                Some(value) => (value, term.clone()),
                None => (T::one(), self.clone()),
            },
            (Operation::Negate, [term]) => (-T::one(), term.clone()),
            _ => (T::one(), self.clone()),
        }
    }

    fn scale_term(coefficient: T, term: Self) -> Self {
        if coefficient == T::one() {
            term
        } else if coefficient == -T::one() {
            Self::new_negate(term)
        } else {
            Self::new_multiply(Self::new_const(coefficient), term)
        }
    }

    fn simplify_product(left: Self, right: Self) -> Self {
        match (left.constant_value(), right.constant_value()) {
            (Some(left), Some(right)) => Self::new_const(left * right),
            (Some(value), _) if value == T::zero() => Self::new_const(T::zero()),
            (_, Some(value)) if value == T::zero() => Self::new_const(T::zero()),
            (Some(value), _) if value == T::one() => right,
            (_, Some(value)) if value == T::one() => left,
            (Some(value), _) if value == -T::one() => Self::simplify_negation(right),
            (_, Some(value)) if value == -T::one() => Self::simplify_negation(left),
            (Some(value), _) => {
                let (nested_coefficient, term) = right.coefficient_and_term();
                if nested_coefficient == T::one() {
                    Self::new_multiply(Self::new_const(value), right)
                } else {
                    Self::scale_term(value * nested_coefficient, term)
                }
            }
            (_, Some(value)) => Self::simplify_product(Self::new_const(value), left),
            _ => Self::new_multiply(left, right),
        }
    }

    fn simplify_quotient(numerator: Self, denominator: Self) -> Self {
        match (numerator.constant_value(), denominator.constant_value()) {
            (Some(numerator), Some(denominator)) => Self::new_const(numerator / denominator),
            (Some(value), _) if value == T::zero() => Self::new_const(T::zero()),
            _ if numerator == denominator => Self::new_const(T::one()),
            (_, Some(value)) if value == T::one() => numerator,
            (_, Some(value)) if value == -T::one() => Self::simplify_negation(numerator),
            _ => Self::new_divide(numerator, denominator),
        }
    }

    fn simplify_power(base: Self, exponent: Self) -> Self {
        if let Some(exponent_value) = exponent.constant_value() {
            if exponent_value == T::zero() {
                return Self::new_const(T::one());
            }
            if exponent_value == T::one() {
                return base;
            }

            if let (Operation::Power, [inner_base, inner_exponent]) =
                (&base.operation, base.operands.as_slice())
            {
                if let Some(inner_exponent) = inner_exponent.constant_value() {
                    return Self::simplify_power(
                        inner_base.clone(),
                        Self::new_const(inner_exponent * exponent_value),
                    );
                }
            }

            if let Some(base_value) = base.constant_value() {
                return Self::new_const(base_value.powf(exponent_value));
            }
        }

        Self::new_power(base, exponent)
    }

    fn simplify_natural_logarithm(operand: Self) -> Self {
        match (&operand.operation, operand.operands.as_slice()) {
            (Operation::Power, [base, exponent]) => {
                Self::simplify_product(exponent.clone(), Self::new_ln(base.clone()))
            }
            _ => match operand.constant_value() {
                Some(value) => Self::new_const(value.ln()),
                None => Self::new_ln(operand),
            },
        }
    }

    fn simplify_common_logarithm(operand: Self) -> Self {
        match (&operand.operation, operand.operands.as_slice()) {
            (Operation::Power, [base, exponent]) => {
                Self::simplify_product(exponent.clone(), Self::new_log10(base.clone()))
            }
            _ => match operand.constant_value() {
                Some(value) => Self::new_const(value.log10()),
                None => Self::new_log10(operand),
            },
        }
    }

    fn simplify_negation(operand: Self) -> Self {
        if let Some(value) = operand.constant_value() {
            return Self::new_const(-value);
        }

        match (&operand.operation, operand.operands.as_slice()) {
            (Operation::Negate, [inner]) => inner.clone(),
            _ => Self::new_negate(operand),
        }
    }

    fn constant_value(&self) -> Option<T> {
        match self.operation {
            Operation::Constant(value) => Some(value),
            _ => None,
        }
    }

    fn sign_value(value: T) -> T {
        if value.is_nan() {
            value
        } else if value > T::zero() {
            T::one()
        } else if value < T::zero() {
            -T::one()
        } else {
            T::zero()
        }
    }

    pub fn evaluate(&self, args: &HashMap<&str, T>) -> Result<T, EvaluationError<'a, T>> {
        match &self.operation {
            Operation::Variable(name) => args
                .get(*name)
                .copied()
                .ok_or(EvaluationError::VariableNotDefined(name)),
            Operation::Constant(value) => Ok(*value),
            Operation::Add => {
                let (left, right) = self.binary_operands(Operation::Add)?;
                Ok(left.evaluate(args)? + right.evaluate(args)?)
            }
            Operation::Subtract => {
                let (left, right) = self.binary_operands(Operation::Subtract)?;
                Ok(left.evaluate(args)? - right.evaluate(args)?)
            }
            Operation::Multiply => {
                let (left, right) = self.binary_operands(Operation::Multiply)?;
                Ok(left.evaluate(args)? * right.evaluate(args)?)
            }
            Operation::Divide => {
                let (numerator, denominator) = self.binary_operands(Operation::Divide)?;
                Ok(numerator.evaluate(args)? / denominator.evaluate(args)?)
            }
            Operation::Power => {
                let (base, exponent) = self.binary_operands(Operation::Power)?;
                Ok(base.evaluate(args)?.powf(exponent.evaluate(args)?))
            }
            Operation::Negate => {
                let (operand,) = self.unary_operands(Operation::Negate)?;
                Ok(-operand.evaluate(args)?)
            }
            Operation::Sine => {
                let (operand,) = self.unary_operands(Operation::Sine)?;
                Ok(operand.evaluate(args)?.sin())
            }
            Operation::Cosine => {
                let (operand,) = self.unary_operands(Operation::Cosine)?;
                Ok(operand.evaluate(args)?.cos())
            }
            Operation::Tangent => {
                let (operand,) = self.unary_operands(Operation::Tangent)?;
                Ok(operand.evaluate(args)?.tan())
            }
            Operation::ArcSine => {
                let (operand,) = self.unary_operands(Operation::ArcSine)?;
                Ok(operand.evaluate(args)?.asin())
            }
            Operation::ArcCosine => {
                let (operand,) = self.unary_operands(Operation::ArcCosine)?;
                Ok(operand.evaluate(args)?.acos())
            }
            Operation::ArcTangent => {
                let (operand,) = self.unary_operands(Operation::ArcTangent)?;
                Ok(operand.evaluate(args)?.atan())
            }
            Operation::HyperbolicSine => {
                let (operand,) = self.unary_operands(Operation::HyperbolicSine)?;
                Ok(operand.evaluate(args)?.sinh())
            }
            Operation::HyperbolicCosine => {
                let (operand,) = self.unary_operands(Operation::HyperbolicCosine)?;
                Ok(operand.evaluate(args)?.cosh())
            }
            Operation::HyperbolicTangent => {
                let (operand,) = self.unary_operands(Operation::HyperbolicTangent)?;
                Ok(operand.evaluate(args)?.tanh())
            }
            Operation::Sign => {
                let (operand,) = self.unary_operands(Operation::Sign)?;
                Ok(Self::sign_value(operand.evaluate(args)?))
            }
            Operation::AbsoluteValue => {
                let (operand,) = self.unary_operands(Operation::AbsoluteValue)?;
                Ok(operand.evaluate(args)?.abs())
            }
            Operation::Exponent => {
                let (operand,) = self.unary_operands(Operation::Exponent)?;
                Ok(operand.evaluate(args)?.exp())
            }
            Operation::SquareRoot => {
                let (operand,) = self.unary_operands(Operation::SquareRoot)?;
                Ok(operand.evaluate(args)?.sqrt())
            }
            Operation::NaturalLogarithm => {
                let (operand,) = self.unary_operands(Operation::NaturalLogarithm)?;
                Ok(operand.evaluate(args)?.ln())
            }
            Operation::CommonLogarithm => {
                let (operand,) = self.unary_operands(Operation::CommonLogarithm)?;
                Ok(operand.evaluate(args)?.log10())
            }
            Operation::ImmediateIf => {
                let (check, if_less_than_zero, otherwise) =
                    self.ternary_operands(Operation::ImmediateIf)?;
                if check.evaluate(args)? < T::zero() {
                    if_less_than_zero.evaluate(args)
                } else {
                    otherwise.evaluate(args)
                }
            }
        }
    }

    fn unary_operands(
        &self,
        operation: Operation<'a, T>,
    ) -> Result<(&Self,), EvaluationError<'a, T>> {
        match self.operands.as_slice() {
            [operand] => Ok((operand,)),
            operands => Err(EvaluationError::InvalidOperandCount {
                operation,
                expected: 1,
                actual: operands.len(),
            }),
        }
    }

    fn binary_operands(
        &self,
        operation: Operation<'a, T>,
    ) -> Result<(&Self, &Self), EvaluationError<'a, T>> {
        match self.operands.as_slice() {
            [left, right] => Ok((left, right)),
            operands => Err(EvaluationError::InvalidOperandCount {
                operation,
                expected: 2,
                actual: operands.len(),
            }),
        }
    }

    fn ternary_operands(
        &self,
        operation: Operation<'a, T>,
    ) -> Result<(&Self, &Self, &Self), EvaluationError<'a, T>> {
        match self.operands.as_slice() {
            [first, second, third] => Ok((first, second, third)),
            operands => Err(EvaluationError::InvalidOperandCount {
                operation,
                expected: 3,
                actual: operands.len(),
            }),
        }
    }
}

impl<T: Float> Display for Operation<'_, T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Variable(name) => write!(formatter, "{name}"),
            Operation::Constant(_) => write!(formatter, "Constant"),
            Operation::Add => write!(formatter, "+"),
            Operation::Subtract => write!(formatter, "-"),
            Operation::Multiply => write!(formatter, "*"),
            Operation::Divide => write!(formatter, "/"),
            Operation::Power => write!(formatter, "^"),
            Operation::Negate => write!(formatter, "-"),
            Operation::Sine => write!(formatter, "sin"),
            Operation::Cosine => write!(formatter, "cos"),
            Operation::Tangent => write!(formatter, "tan"),
            Operation::ArcSine => write!(formatter, "asin"),
            Operation::ArcCosine => write!(formatter, "acos"),
            Operation::ArcTangent => write!(formatter, "atan"),
            Operation::HyperbolicSine => write!(formatter, "sinh"),
            Operation::HyperbolicCosine => write!(formatter, "cosh"),
            Operation::HyperbolicTangent => write!(formatter, "tanh"),
            Operation::Sign => write!(formatter, "sign"),
            Operation::AbsoluteValue => write!(formatter, "abs"),
            Operation::Exponent => write!(formatter, "exp"),
            Operation::SquareRoot => write!(formatter, "sqrt"),
            Operation::NaturalLogarithm => write!(formatter, "ln"),
            Operation::CommonLogarithm => write!(formatter, "log10"),
            Operation::ImmediateIf => write!(formatter, "iif"),
        }
    }
}

impl<T: Float + Display> Display for MathExpr<'_, T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.operation, self.operands.as_slice()) {
            (Operation::Variable(name), []) => write!(formatter, "{name}"),
            (Operation::Constant(value), []) => write!(formatter, "{value}"),
            (Operation::Add, [left, right]) => write!(formatter, "({left} + {right})"),
            (Operation::Subtract, [left, right]) => write!(formatter, "({left} - {right})"),
            (Operation::Multiply, [left, right]) => write!(formatter, "({left} * {right})"),
            (Operation::Divide, [numerator, denominator]) => {
                write!(formatter, "({numerator} / {denominator})")
            }
            (Operation::Power, [base, exponent]) => {
                write!(formatter, "({base} ^ {exponent})")
            }
            (Operation::Negate, [operand]) => write!(formatter, "(-{operand})"),
            (Operation::Sine, [operand]) => write!(formatter, "sin({operand})"),
            (Operation::Cosine, [operand]) => write!(formatter, "cos({operand})"),
            (Operation::Tangent, [operand]) => write!(formatter, "tan({operand})"),
            (Operation::ArcSine, [operand]) => write!(formatter, "asin({operand})"),
            (Operation::ArcCosine, [operand]) => write!(formatter, "acos({operand})"),
            (Operation::ArcTangent, [operand]) => write!(formatter, "atan({operand})"),
            (Operation::HyperbolicSine, [operand]) => write!(formatter, "sinh({operand})"),
            (Operation::HyperbolicCosine, [operand]) => write!(formatter, "cosh({operand})"),
            (Operation::HyperbolicTangent, [operand]) => write!(formatter, "tanh({operand})"),
            (Operation::Sign, [operand]) => write!(formatter, "sign({operand})"),
            (Operation::AbsoluteValue, [operand]) => write!(formatter, "abs({operand})"),
            (Operation::Exponent, [operand]) => write!(formatter, "exp({operand})"),
            (Operation::SquareRoot, [operand]) => write!(formatter, "sqrt({operand})"),
            (Operation::NaturalLogarithm, [operand]) => write!(formatter, "ln({operand})"),
            (Operation::CommonLogarithm, [operand]) => write!(formatter, "log10({operand})"),
            (Operation::ImmediateIf, [check, if_less_than_zero, otherwise]) => {
                write!(formatter, "iif({check}, {if_less_than_zero}, {otherwise})")
            }
            _ => Err(std::fmt::Error),
        }
    }
}

impl<T: Float> Display for EvaluationError<'_, T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::InvalidOperandCount {
                operation,
                expected,
                actual,
            } => write!(
                formatter,
                "{operation} expects {expected} operands, but received {actual}"
            ),
            EvaluationError::VariableNotDefined(name) => {
                write!(formatter, "Variable {name} not defined")
            }
        }
    }
}

// impl<T: Float> Error for EvaluationError<T> {}

#[cfg(test)]
mod tests;
