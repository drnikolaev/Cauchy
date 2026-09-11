use cauchy::fortran::evaluate_math_expr_from_fortran;

#[test]
fn fortran_calls_rust_evaluate_and_returns_answer_by_reference() {
    let answer = evaluate_math_expr_from_fortran("2 + 3 * x", &[5.0]).unwrap();

    assert_eq!(answer, 17.0);
}

#[test]
fn binds_each_vector_component() {
    let answer = evaluate_math_expr_from_fortran("x0 + 3*x1", &[2.0, 5.0]).unwrap();
    assert_eq!(answer, 17.0);
}

#[test]
fn constant_expression_accepts_empty_state() {
    assert_eq!(evaluate_math_expr_from_fortran("2+3*5", &[]), Ok(17.0));
}

#[test]
fn undefined_component_returns_evaluation_error() {
    assert_eq!(evaluate_math_expr_from_fortran("x1", &[5.0]), Err(2));
}
