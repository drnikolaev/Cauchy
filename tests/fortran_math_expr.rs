use cauchy::fortran::evaluate_math_expr_from_fortran;

#[test]
fn fortran_calls_rust_evaluate_and_returns_answer_by_reference() {
    let answer = evaluate_math_expr_from_fortran("2 + 3 * x", &[5.0]).unwrap();

    assert_eq!(answer, 17.0);
}
