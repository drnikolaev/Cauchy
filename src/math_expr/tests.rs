use super::{EvaluationError, MathExpr, Operation};
use std::collections::HashMap;

#[test]
fn evaluate_var() {
    let expression = MathExpr::<f64>::new_var("x");
    let val = expression.evaluate(&HashMap::from([("x", 1.0)]));
    assert_eq!(val, Ok(1.0));
}

#[test]
fn reject_undefined_var() {
    let expression = MathExpr::<f64>::new_var("x");

    assert_eq!(
        expression.evaluate(&HashMap::new()),
        Err(EvaluationError::VariableNotDefined("x"))
    );
}

#[test]
fn evaluate_addition() {
    let expression = MathExpr::new_add(MathExpr::new_const(1.5), MathExpr::new_const(2.5));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(4.0));
}

#[test]
fn evaluate_subtraction() {
    let expression = MathExpr::new_subtract(MathExpr::new_const(1.5), MathExpr::new_const(2.5));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(-1.0));
}

#[test]
fn evaluate_multiplication() {
    let expression = MathExpr::new_multiply(MathExpr::new_const(1.5_f64), MathExpr::new_const(2.0));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(3.0));

    let expression = MathExpr::new_multiply(MathExpr::new_const(1.5_f32), MathExpr::new_const(2.0));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(3.0));
}

#[test]
fn evaluate_division() {
    let expression = MathExpr::new_divide(MathExpr::new_const(3.0_f64), MathExpr::new_const(2.0));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(1.5));
    assert_eq!(expression.to_string(), "(3 / 2)");

    let expression = MathExpr::new_divide(MathExpr::new_const(3.0_f32), MathExpr::new_const(2.0));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(1.5));
}

#[test]
fn evaluate_power() {
    let expression = MathExpr::new_power(MathExpr::new_const(2.0_f64), MathExpr::new_const(3.0));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(8.0));

    let expression = MathExpr::new_power(MathExpr::new_const(4.0_f32), MathExpr::new_const(-0.5));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.5));
}

#[test]
fn derive_power() {
    let expression = MathExpr::new_power(MathExpr::<f64>::new_var("x"), MathExpr::new_const(3.0));
    let derivative = expression.derive("x").simplify();

    assert_eq!(derivative.evaluate(&HashMap::from([("x", 2.0)])), Ok(12.0));
}

#[test]
fn derive_division() {
    let variable = MathExpr::<f64>::new_var("x");
    let expression = MathExpr::new_divide(
        variable.clone(),
        MathExpr::new_add(variable, MathExpr::new_const(1.0)),
    );
    let derivative = expression.derive("x");

    assert_eq!(
        derivative.evaluate(&HashMap::from([("x", 3.0)])),
        Ok(1.0 / 16.0)
    );
}

#[test]
fn derive_multiplication() {
    let variable = MathExpr::<f64>::new_var("x");
    let expression = MathExpr::new_multiply(variable.clone(), variable);
    let derivative = expression.derive("x");

    assert_eq!(derivative.evaluate(&HashMap::from([("x", 3.0)])), Ok(6.0));
}

#[test]
fn evaluate_negation() {
    let expression = MathExpr::new_negate(MathExpr::new_const(1.5_f64));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(-1.5));

    let expression = MathExpr::new_negate(MathExpr::new_const(1.5_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(-1.5));
}

#[test]
fn derive_negation() {
    let expression = MathExpr::<f64>::new_negate(MathExpr::new_var("x"));
    let derivative = expression.derive("x");

    assert_eq!(derivative.evaluate(&HashMap::new()), Ok(-1.0));
}

#[test]
fn reject_invalid_operand_count() {
    let expression = MathExpr {
        operation: Operation::Subtract,
        operands: vec![MathExpr::new_const(1.0)],
    };
    assert_eq!(
        expression.evaluate(&HashMap::new()),
        Err(EvaluationError::InvalidOperandCount {
            operation: Operation::Subtract,
            expected: 2,
            actual: 1,
        })
    );
}

#[test]
fn derive_sin() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_multiply(variable.clone(), variable);
    let derivative = MathExpr::new_sin(square).derive("x");
    let x = 0.5_f64;
    let expected = 2.0 * x * (x * x).cos();
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn derive_cos() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_multiply(variable.clone(), variable);
    let derivative = MathExpr::new_cos(square).derive("x");
    let x = 0.5_f64;
    let expected = -2.0 * x * (x * x).sin();
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_tan() {
    let expression = MathExpr::new_tan(MathExpr::new_const(std::f64::consts::FRAC_PI_4));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0).abs() < 1e-12);

    let expression = MathExpr::new_tan(MathExpr::new_const(0.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.0));
}

#[test]
fn derive_tan() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_tan(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = 2.0 * x / (x * x).cos().powi(2);
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_asin() {
    let expression = MathExpr::new_asin(MathExpr::new_const(1.0_f64));
    assert!(
        (expression.evaluate(&HashMap::new()).unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-12
    );

    let expression = MathExpr::new_asin(MathExpr::new_const(0.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.0));
}

#[test]
fn derive_asin() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_asin(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = 2.0 * x / (1.0 - x.powi(4)).sqrt();
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_acos() {
    let expression = MathExpr::new_acos(MathExpr::new_const(0.0_f64));
    assert!(
        (expression.evaluate(&HashMap::new()).unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-12
    );

    let expression = MathExpr::new_acos(MathExpr::new_const(1.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.0));
}

#[test]
fn derive_acos() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_acos(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = -2.0 * x / (1.0 - x.powi(4)).sqrt();
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_atan() {
    let expression = MathExpr::new_atan(MathExpr::new_const(1.0_f64));
    assert!(
        (expression.evaluate(&HashMap::new()).unwrap() - std::f64::consts::FRAC_PI_4).abs() < 1e-12
    );

    let expression = MathExpr::new_atan(MathExpr::new_const(0.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.0));
}

#[test]
fn derive_atan() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_atan(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = 2.0 * x / (1.0 + x.powi(4));
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_sinh() {
    let expression = MathExpr::new_sinh(MathExpr::new_const(1.0_f64));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0_f64.sinh()).abs() < 1e-12);

    let expression = MathExpr::new_sinh(MathExpr::new_const(0.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.0));
}

#[test]
fn derive_sinh() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_sinh(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = 2.0 * x * (x * x).cosh();
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_cosh() {
    let expression = MathExpr::new_cosh(MathExpr::new_const(1.0_f64));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0_f64.cosh()).abs() < 1e-12);

    let expression = MathExpr::new_cosh(MathExpr::new_const(0.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(1.0));
}

#[test]
fn derive_cosh() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_cosh(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = 2.0 * x * (x * x).sinh();
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_tanh() {
    let expression = MathExpr::new_tanh(MathExpr::new_const(1.0_f64));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0_f64.tanh()).abs() < 1e-12);

    let expression = MathExpr::new_tanh(MathExpr::new_const(0.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.0));
}

#[test]
fn derive_tanh() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_tanh(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = 2.0 * x / (x * x).cosh().powi(2);
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_sign() {
    let negative = MathExpr::new_sign(MathExpr::new_const(-3.0_f64));
    let zero = MathExpr::new_sign(MathExpr::new_const(0.0_f64));
    let positive = MathExpr::new_sign(MathExpr::new_const(2.0_f32));

    assert_eq!(negative.evaluate(&HashMap::new()), Ok(-1.0));
    assert_eq!(zero.evaluate(&HashMap::new()), Ok(0.0));
    assert_eq!(positive.evaluate(&HashMap::new()), Ok(1.0));
}

#[test]
fn derive_sign() {
    let expression = MathExpr::new_sign(MathExpr::<f64>::new_var("x"));

    assert_eq!(expression.derive("x").to_string(), "0");
}

#[test]
fn evaluate_abs() {
    let negative = MathExpr::new_abs(MathExpr::new_const(-3.0_f64));
    let positive = MathExpr::new_abs(MathExpr::new_const(2.0_f32));

    assert_eq!(negative.evaluate(&HashMap::new()), Ok(3.0));
    assert_eq!(positive.evaluate(&HashMap::new()), Ok(2.0));
}

#[test]
fn derive_abs() {
    let variable = MathExpr::<f64>::new_var("x");
    let square_minus_one = MathExpr::new_subtract(
        MathExpr::new_power(variable, MathExpr::new_const(2.0)),
        MathExpr::new_const(1.0),
    );
    let derivative = MathExpr::new_abs(square_minus_one).derive("x").simplify();

    assert_eq!(derivative.evaluate(&HashMap::from([("x", 2.0)])), Ok(4.0));
    assert_eq!(derivative.evaluate(&HashMap::from([("x", 0.5)])), Ok(-1.0));
}

#[test]
fn evaluate_exp() {
    let expression = MathExpr::new_exp(MathExpr::new_const(1.0_f64));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - std::f64::consts::E).abs() < 1e-12);

    let expression = MathExpr::new_exp(MathExpr::new_const(0.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(1.0));
}

#[test]
fn derive_exp() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_exp(square).derive("x").simplify();
    let x = 0.5_f64;
    let expected = 2.0 * x * (x * x).exp();
    let actual = derivative.evaluate(&HashMap::from([("x", x)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn evaluate_sqrt() {
    let expression = MathExpr::new_sqrt(MathExpr::new_const(9.0_f64));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(3.0));

    let expression = MathExpr::new_sqrt(MathExpr::new_const(2.25_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(1.5));
}

#[test]
fn derive_sqrt() {
    let expression = MathExpr::new_sqrt(MathExpr::<f64>::new_var("x"));
    let derivative = expression.derive("x").simplify();

    assert_eq!(derivative.evaluate(&HashMap::from([("x", 4.0)])), Ok(0.25));
}

#[test]
fn evaluate_ln() {
    let expression = MathExpr::new_ln(MathExpr::new_const(std::f64::consts::E));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0).abs() < 1e-12);

    let expression = MathExpr::new_ln(MathExpr::new_const(1.0_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(0.0));
}

#[test]
fn derive_ln() {
    let variable = MathExpr::<f64>::new_var("x");
    let square = MathExpr::new_power(variable, MathExpr::new_const(2.0));
    let derivative = MathExpr::new_ln(square).derive("x").simplify();

    assert_eq!(derivative.evaluate(&HashMap::from([("x", 2.0)])), Ok(1.0));
}

#[test]
fn evaluate_log10() {
    let expression = MathExpr::new_log10(MathExpr::new_const(1000.0_f64));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(3.0));

    let expression = MathExpr::new_log10(MathExpr::new_const(0.1_f32));
    assert_eq!(expression.evaluate(&HashMap::new()), Ok(-1.0));
}

#[test]
fn derive_log10() {
    let expression = MathExpr::new_log10(MathExpr::<f64>::new_var("x"));
    let derivative = expression.derive("x").simplify();
    let expected = 1.0 / (10.0_f64 * 10.0_f64.ln());
    let actual = derivative.evaluate(&HashMap::from([("x", 10.0)])).unwrap();

    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn expression_to_string() {
    let expression = MathExpr::new_subtract(
        MathExpr::new_add(MathExpr::<f64>::new_var("x"), MathExpr::new_const(2.0)),
        MathExpr::new_multiply(
            MathExpr::new_negate(MathExpr::new_sin(MathExpr::new_var("y"))),
            MathExpr::new_cos(MathExpr::new_var("z")),
        ),
    );

    assert_eq!(expression.to_string(), "((x + 2) - ((-sin(y)) * cos(z)))");
}

#[test]
fn parse_to_string_output() {
    let original = MathExpr::new_divide(
        MathExpr::new_subtract(
            MathExpr::new_sin(MathExpr::<f64>::new_var("x")),
            MathExpr::new_cos(MathExpr::new_var("y")),
        ),
        MathExpr::new_add(
            MathExpr::new_const(2.0),
            MathExpr::new_negate(MathExpr::new_var("z")),
        ),
    );
    let source = original.to_string();
    let parsed = MathExpr::<f64>::parse(&source).unwrap();
    let args = HashMap::from([("x", 0.5), ("y", 1.0), ("z", 0.25)]);

    assert_eq!(parsed.to_string(), source);
    assert_eq!(parsed.evaluate(&args), original.evaluate(&args));
}

#[test]
fn parse_and_evaluate_complex_expression() {
    let source = "sqrt(x ^ 2 + y ^ 2) + sin(theta) * exp(ln(scale)) - abs(offset) / log10(100)";
    let expression = MathExpr::<f64>::parse(source).unwrap();
    let args = HashMap::from([
        ("x", 3.0),
        ("y", 4.0),
        ("theta", std::f64::consts::FRAC_PI_2),
        ("scale", 4.0),
        ("offset", -3.0),
    ]);

    let result = expression.evaluate(&args).unwrap();

    assert!((result - 7.5).abs() < 1e-12);
}

#[test]
fn parse_and_evaluate_complex_expression_derivatives() {
    let source = "sqrt(x ^ 2 + Y ^ 2) + sin(theta) * exp(ln(scale)) - abs(offset) / log10(100)";
    let expression = MathExpr::<f64>::parse(source).unwrap();
    let args = HashMap::from([
        ("x", 3.0),
        ("Y", 4.0),
        ("theta", std::f64::consts::FRAC_PI_2),
        ("scale", 4.0),
        ("offset", -3.0),
    ]);

    let dx = expression.derive("x").simplify();
    let dy = expression.derive("Y").simplify();
    let derivative_by_x = dx.evaluate(&args).unwrap();
    let derivative_by_y = dy.evaluate(&args).unwrap();
    println!("{:?}", dx);
    println!("{:?}", dy);
    assert!((derivative_by_x - 0.6).abs() < 1e-12);
    assert!((derivative_by_y - 0.8).abs() < 1e-12);
}

#[test]
fn parse_operator_precedence() {
    let parsed = MathExpr::<f64>::parse("x + 2 * -y").unwrap();

    assert_eq!(parsed.to_string(), "(x + (2 * (-y)))");
}

#[test]
fn parse_power_precedence_and_formatting() {
    let parsed = MathExpr::<f64>::parse("-x ^ 2 * y + z ^ -2").unwrap();

    assert_eq!(parsed.to_string(), "(((-(x ^ 2)) * y) + (z ^ -2))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_exp_and_formatting() {
    let parsed = MathExpr::<f64>::parse("exp(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "exp((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_tan_and_formatting() {
    let parsed = MathExpr::<f64>::parse("tan(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "tan((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_asin_and_formatting() {
    let parsed = MathExpr::<f64>::parse("asin(x / 2)").unwrap();

    assert_eq!(parsed.to_string(), "asin((x / 2))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_acos_and_formatting() {
    let parsed = MathExpr::<f64>::parse("acos(x / 2)").unwrap();

    assert_eq!(parsed.to_string(), "acos((x / 2))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_atan_and_formatting() {
    let parsed = MathExpr::<f64>::parse("atan(x / 2)").unwrap();

    assert_eq!(parsed.to_string(), "atan((x / 2))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_sinh_and_formatting() {
    let parsed = MathExpr::<f64>::parse("sinh(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "sinh((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_cosh_and_formatting() {
    let parsed = MathExpr::<f64>::parse("cosh(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "cosh((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_tanh_and_formatting() {
    let parsed = MathExpr::<f64>::parse("tanh(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "tanh((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_sign_and_formatting() {
    let parsed = MathExpr::<f64>::parse("sign(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "sign((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_abs_and_formatting() {
    let parsed = MathExpr::<f64>::parse("abs(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "abs((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_sqrt_and_formatting() {
    let parsed = MathExpr::<f64>::parse("sqrt(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "sqrt((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_ln_and_formatting() {
    let parsed = MathExpr::<f64>::parse("ln(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "ln((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_log10_and_formatting() {
    let parsed = MathExpr::<f64>::parse("log10(x + 1)").unwrap();

    assert_eq!(parsed.to_string(), "log10((x + 1))");
    assert_eq!(MathExpr::<f64>::parse(&parsed.to_string()).unwrap(), parsed);
}

#[test]
fn parse_and_evaluate_iif() {
    let expression = MathExpr::<f64>::parse("iif(x, -x, x ^ 2)").unwrap();

    assert_eq!(expression.to_string(), "iif(x, (-x), (x ^ 2))");
    assert_eq!(expression.evaluate(&HashMap::from([("x", -3.0)])), Ok(3.0));
    assert_eq!(expression.evaluate(&HashMap::from([("x", 2.0)])), Ok(4.0));
    assert_eq!(expression.evaluate(&HashMap::from([("x", 0.0)])), Ok(0.0));
}

#[test]
fn evaluate_iif_lazily() {
    let expression = MathExpr::<f64>::parse("iif(x, undefined, 42)").unwrap();

    assert_eq!(expression.evaluate(&HashMap::from([("x", 1.0)])), Ok(42.0));
}

#[test]
fn simplify_iif_with_constant_condition() {
    assert_eq!(
        MathExpr::<f64>::parse("iif(-1, x, undefined)")
            .unwrap()
            .simplify()
            .to_string(),
        "x"
    );
    assert_eq!(
        MathExpr::<f64>::parse("iif(0, undefined, x)")
            .unwrap()
            .simplify()
            .to_string(),
        "x"
    );
}

#[test]
fn simplify_repeated_terms() {
    let expression = MathExpr::<f64>::parse("x + x + x + x").unwrap();
    let simplified = expression.simplify();

    assert_eq!(simplified.to_string(), "(4 * x)");
    assert_eq!(simplified.evaluate(&HashMap::from([("x", 3.0)])), Ok(12.0));
}

#[test]
fn simplify_like_terms_and_constants() {
    let expression = MathExpr::<f64>::parse("2*x + 3*x - x + 2 - 2").unwrap();
    let simplified = expression.simplify();

    assert_eq!(simplified.to_string(), "(4 * x)");
}

#[test]
fn simplify_subtracting_identical_variables() {
    let expression = MathExpr::<f64>::parse("x - x").unwrap();

    assert_eq!(expression.simplify().to_string(), "0");
}

#[test]
fn simplify_dividing_identical_variables() {
    let expression = MathExpr::<f64>::parse("x / x").unwrap();

    assert_eq!(expression.simplify().to_string(), "1");
}

#[test]
fn simplify_numeric_operations() {
    let expression = MathExpr::<f64>::parse("sin(0) + cos(0) * 1").unwrap();

    assert_eq!(expression.simplify().to_string(), "1");
}

#[test]
fn simplify_zero_quotient_cascades_through_expression() {
    let expression = MathExpr::<f64>::parse(
        "(((2 * x) / (2 * sqrt(((x ^ 2) + (Y ^ 2))))) + (sin(theta) * ((0 / scale) * exp(ln(scale)))))",
    )
    .unwrap();

    assert_eq!(
        expression.simplify().to_string(),
        "((2 * x) / (2 * sqrt(((x ^ 2) + (Y ^ 2)))))"
    );
}

#[test]
fn simplify_power() {
    assert_eq!(
        MathExpr::<f64>::parse("x ^ 0")
            .unwrap()
            .simplify()
            .to_string(),
        "1"
    );
    assert_eq!(
        MathExpr::<f64>::parse("x ^ 1")
            .unwrap()
            .simplify()
            .to_string(),
        "x"
    );
    assert_eq!(
        MathExpr::<f64>::parse("2 ^ 3")
            .unwrap()
            .simplify()
            .to_string(),
        "8"
    );
}

#[test]
fn simplify_nested_power() {
    let expression = MathExpr::<f64>::parse("(x ^ 3) ^ 2").unwrap();

    assert_eq!(expression.simplify().to_string(), "(x ^ 6)");
}

#[test]
fn simplify_logarithm_of_power() {
    let expression = MathExpr::<f64>::parse("ln(x ^ y)").unwrap();

    assert_eq!(expression.simplify().to_string(), "(y * ln(x))");
}

#[test]
fn simplify_common_logarithm_of_power() {
    let expression = MathExpr::<f64>::parse("log10(x ^ y)").unwrap();

    assert_eq!(expression.simplify().to_string(), "(y * log10(x))");
}

#[test]
fn simplify_exp_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("exp(0)")
            .unwrap()
            .simplify()
            .to_string(),
        "1"
    );
}

#[test]
fn simplify_tan_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("tan(0)")
            .unwrap()
            .simplify()
            .to_string(),
        "0"
    );
}

#[test]
fn simplify_asin_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("asin(0)")
            .unwrap()
            .simplify()
            .to_string(),
        "0"
    );
}

#[test]
fn simplify_acos_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("acos(1)")
            .unwrap()
            .simplify()
            .to_string(),
        "0"
    );
}

#[test]
fn simplify_atan_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("atan(0)")
            .unwrap()
            .simplify()
            .to_string(),
        "0"
    );
}

#[test]
fn simplify_sinh_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("sinh(0)")
            .unwrap()
            .simplify()
            .to_string(),
        "0"
    );
}

#[test]
fn simplify_cosh_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("cosh(0)")
            .unwrap()
            .simplify()
            .to_string(),
        "1"
    );
}

#[test]
fn simplify_tanh_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("tanh(0)")
            .unwrap()
            .simplify()
            .to_string(),
        "0"
    );
}

#[test]
fn simplify_sign_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("sign(-42)")
            .unwrap()
            .simplify()
            .to_string(),
        "-1"
    );
}

#[test]
fn simplify_abs_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("abs(-42)")
            .unwrap()
            .simplify()
            .to_string(),
        "42"
    );
}

#[test]
fn simplify_sqrt_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("sqrt(16)")
            .unwrap()
            .simplify()
            .to_string(),
        "4"
    );
}

#[test]
fn simplify_ln_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("ln(1)")
            .unwrap()
            .simplify()
            .to_string(),
        "0"
    );
}

#[test]
fn simplify_log10_constant() {
    assert_eq!(
        MathExpr::<f64>::parse("log10(100)")
            .unwrap()
            .simplify()
            .to_string(),
        "2"
    );
}

#[test]
fn simplify_derivative() {
    let variable = MathExpr::<f64>::new_var("x");
    let expression = MathExpr::new_multiply(variable.clone(), variable);

    assert_eq!(expression.derive("x").simplify().to_string(), "(2 * x)");
    assert_eq!(expression.derive("y").simplify().to_string(), "0");
}

#[test]
fn reject_invalid_expression() {
    let error = MathExpr::<f64>::parse("sin(x").unwrap_err();

    assert_eq!(error.position, 5);
    assert_eq!(error.message, "expected ')'");
}

#[test]
fn evaluate_sin() {
    let expression = MathExpr::new_sin(MathExpr::new_const(std::f64::consts::PI / 2.0));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0).abs() < f64::EPSILON);
    let expression = MathExpr::new_sin(MathExpr::new_const(std::f64::consts::PI * 1.5));
    assert!((expression.evaluate(&HashMap::new()).unwrap() + 1.0).abs() < f64::EPSILON);
    let expression = MathExpr::new_sin(MathExpr::new_const(std::f32::consts::PI / 2.0));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0).abs() < f32::EPSILON);
    let expression = MathExpr::new_sin(MathExpr::new_const(std::f32::consts::PI * 1.5));
    assert!((expression.evaluate(&HashMap::new()).unwrap() + 1.0).abs() < f32::EPSILON);
}

#[test]
fn evaluate_sin_add() {
    let expression = MathExpr::new_add(
        MathExpr::new_sin(MathExpr::new_const(std::f64::consts::PI / 2.0)),
        MathExpr::new_const(3.0),
    );
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 4.0).abs() < f64::EPSILON);
}

#[test]
fn evaluate_cos() {
    let expression = MathExpr::new_cos(MathExpr::new_const(0.0_f64));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0).abs() < f64::EPSILON);
    let expression = MathExpr::new_cos(MathExpr::new_const(std::f64::consts::PI));
    assert!((expression.evaluate(&HashMap::new()).unwrap() + 1.0).abs() < f64::EPSILON);
    let expression = MathExpr::new_cos(MathExpr::new_const(0.0_f32));
    assert!((expression.evaluate(&HashMap::new()).unwrap() - 1.0).abs() < f32::EPSILON);
    let expression = MathExpr::new_cos(MathExpr::new_const(std::f32::consts::PI));
    assert!((expression.evaluate(&HashMap::new()).unwrap() + 1.0).abs() < f32::EPSILON);
}
