use cauchy::fortran::matrix_inverse;

#[test]
fn call_fortran_matrix_inverse_from_rust() {
    let matrix = [
        4.0, 7.0, // column 1
        2.0, 6.0, // column 2
    ];

    let inverse = matrix_inverse(&matrix, 2).unwrap();
    let expected = [
        0.6, -0.7, // column 1
        -0.2, 0.4, // column 2
    ];

    for (actual, expected) in inverse.iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-12);
    }
}
