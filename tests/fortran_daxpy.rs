use cauchy::fortran::daxpy;

#[test]
fn call_fortran_daxpy_from_rust() {
    let x = [1.0, 2.0, 3.0, 4.0];
    let mut y = [10.0, 20.0, 30.0, 40.0];

    daxpy(2.5, &x, &mut y);

    assert_eq!(y, [12.5, 25.0, 37.5, 50.0]);
}
