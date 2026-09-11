use crate::math_expr::MathExpr;
use std::collections::HashMap;
use std::ffi::{c_char, c_double, c_int};
use std::fmt::{Display, Formatter};

unsafe extern "C" {
    fn cauchy_daxpy(n: c_int, alpha: c_double, x: *const c_double, y: *mut c_double);
    fn cauchy_fortran_evaluate_math_expr(
        source: *const c_char,
        source_len: c_int,
        x: *const c_double,
        x_len: c_int,
        answer: *mut c_double,
        info: *mut c_int,
    );
    fn cauchy_matrix_exp(
        n: c_int,
        matrix: *const c_double,
        precision: c_double,
        result: *mut c_double,
        work: *mut c_double,
        info: *mut c_int,
    );
    fn cauchy_matrix_inverse(
        n: c_int,
        matrix: *const c_double,
        inverse: *mut c_double,
        pivots: *mut c_int,
        work: *mut c_double,
        work_len: c_int,
        info: *mut c_int,
    );
}

/// Parses and evaluates an expression on behalf of the Fortran wrapper.
///
/// # Safety
///
/// `source` must address `source_len` readable bytes, and `answer` must point
/// to writable storage for one `c_double`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cauchy_evaluate_math_expr(
    source: *const c_char,
    source_len: c_int,
    x: *const c_double,
    x_len: c_int,
    answer: *mut c_double,
) -> c_int {
    if source.is_null() || answer.is_null() || source_len < 0 {
        return -1;
    }

    let bytes = unsafe { std::slice::from_raw_parts(source.cast::<u8>(), source_len as usize) };
    let source = match std::str::from_utf8(bytes) {
        Ok(source) => source,
        Err(_) => return -2,
    };
    let expression = match MathExpr::<f64>::parse(source) {
        Ok(expression) => expression,
        Err(_) => return 1,
    };
    // let x = unsafe { std::slice::from_raw_parts(x, x_len as usize) };
    let args_map = HashMap::new();
    let value = match expression.evaluate(&args_map) {
        Ok(value) => value,
        Err(_) => return 2,
    };

    unsafe { answer.write(value) };
    0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixInverseError {
    Singular { pivot: usize },
    LapackFailure { info: c_int },
}

impl Display for MatrixInverseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Singular { pivot } => {
                write!(formatter, "matrix is singular at one-based pivot {pivot}")
            }
            Self::LapackFailure { info } => {
                write!(formatter, "LAPACK matrix inversion failed with INFO={info}")
            }
        }
    }
}

impl std::error::Error for MatrixInverseError {}

/// Computes `y := alpha * x + y` using the BLAS DAXPY routine.
pub fn daxpy(alpha: f64, x: &[f64], y: &mut [f64]) {
    assert_eq!(x.len(), y.len(), "x and y must have the same length");
    if x.is_empty() {
        return;
    }

    let n = c_int::try_from(x.len()).expect("vectors are too long for the BLAS integer ABI");
    unsafe {
        cauchy_daxpy(n, alpha, x.as_ptr(), y.as_mut_ptr());
    }
}

/// Sends an expression through Fortran and back to Rust for evaluation.
pub fn evaluate_math_expr_from_fortran(source: &str, x: &[f64]) -> Result<f64, c_int> {
    let source_len = c_int::try_from(source.len()).map_err(|_| -3)?;
    let x_len = c_int::try_from(x.len()).map_err(|_| -4)?;
    let mut answer = 0.0;
    let mut info = 0;

    unsafe {
        cauchy_fortran_evaluate_math_expr(
            source.as_ptr().cast::<c_char>(),
            source_len,
            x.as_ptr().cast::<c_double>(),
            x_len,
            &mut answer,
            &mut info,
        );
    }

    match info {
        0 => Ok(answer),
        info => Err(info),
    }
}

/// Computes the exponential of a square matrix in Fortran column-major order.
///
/// `precision` is the requested relative stopping tolerance for the scaled
/// Taylor series. The implementation uses scaling and squaring together with
/// BLAS matrix operations and a LAPACK matrix norm. The returned matrix is
/// also in column-major order.
pub fn matrix_exp(matrix: &[f64], order: usize, precision: f64) -> Vec<f64> {
    let element_count = order
        .checked_mul(order)
        .expect("matrix dimensions overflow usize");
    assert_eq!(
        matrix.len(),
        element_count,
        "matrix must contain order * order elements"
    );
    assert!(
        precision.is_finite() && precision > 0.0,
        "precision must be finite and positive"
    );
    if order == 0 {
        return Vec::new();
    }

    let n = c_int::try_from(order).expect("matrix order exceeds the LAPACK integer ABI");
    c_int::try_from(element_count).expect("matrix is too large for the LAPACK integer ABI");
    let work_len = element_count
        .checked_mul(2)
        .expect("matrix workspace size overflows usize");
    let mut result = vec![0.0; element_count];
    let mut work = vec![0.0; work_len];
    let mut info = 0;

    unsafe {
        cauchy_matrix_exp(
            n,
            matrix.as_ptr(),
            precision,
            result.as_mut_ptr(),
            work.as_mut_ptr(),
            &mut info,
        );
    }

    assert_eq!(
        info, 0,
        "Fortran matrix exponential failed with INFO={info}"
    );
    result
}

/// Inverts a square matrix in Fortran column-major order using pivoted LAPACK
/// LU factorization. The returned matrix is also in column-major order.
pub fn matrix_inverse(matrix: &[f64], order: usize) -> Result<Vec<f64>, MatrixInverseError> {
    let element_count = order
        .checked_mul(order)
        .expect("matrix dimensions overflow usize");
    assert_eq!(
        matrix.len(),
        element_count,
        "matrix must contain order * order elements"
    );
    if order == 0 {
        return Ok(Vec::new());
    }

    let n = c_int::try_from(order).expect("matrix order exceeds the LAPACK integer ABI");
    c_int::try_from(element_count).expect("matrix is too large for the LAPACK integer ABI");
    let work_len = order
        .checked_mul(64)
        .expect("matrix inversion workspace size overflows usize");
    let lapack_work_len =
        c_int::try_from(work_len).expect("workspace exceeds the LAPACK integer ABI");
    let mut inverse = vec![0.0; element_count];
    let mut pivots = vec![0; order];
    let mut work = vec![0.0; work_len];
    let mut info = 0;

    unsafe {
        cauchy_matrix_inverse(
            n,
            matrix.as_ptr(),
            inverse.as_mut_ptr(),
            pivots.as_mut_ptr(),
            work.as_mut_ptr(),
            lapack_work_len,
            &mut info,
        );
    }

    match info {
        0 => Ok(inverse),
        pivot if pivot > 0 => Err(MatrixInverseError::Singular {
            pivot: pivot as usize,
        }),
        info => Err(MatrixInverseError::LapackFailure { info }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MatrixInverseError, daxpy, evaluate_math_expr_from_fortran, matrix_exp, matrix_inverse,
    };

    #[test]
    fn daxpy_updates_output_vector() {
        let x = [2.0, -1.0, 4.0];
        let mut y = [1.0, 3.0, -2.0];

        daxpy(-0.5, &x, &mut y);

        assert_eq!(y, [0.0, 3.5, -4.0]);
    }

    #[test]
    fn fortran_calls_rust_math_expr_evaluate() {
        let answer = evaluate_math_expr_from_fortran("2 + 3 * 5",&[5.0]).unwrap();

        assert_eq!(answer, 17.0);
    }

    #[test]
    fn matrix_exp_computes_diagonal_exponential() {
        let matrix = [1.0, 0.0, 0.0, -1.0];

        let result = matrix_exp(&matrix, 2, 1e-14);

        assert!((result[0] - std::f64::consts::E).abs() < 1e-12);
        assert_eq!(result[1], 0.0);
        assert_eq!(result[2], 0.0);
        assert!((result[3] - 1.0 / std::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn matrix_exp_supports_column_major_nondiagonal_matrices() {
        let matrix = [
            0.0, 1.0, // column 1
            0.0, 0.0, // column 2
        ];

        let result = matrix_exp(&matrix, 2, 1e-14);

        assert_eq!(result, [1.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn matrix_inverse_has_small_identity_residual() {
        let matrix = [4.0, 7.0, 2.0, 3.0, 6.0, 1.0, 2.0, 5.0, 3.0];

        let inverse = matrix_inverse(&matrix, 3).unwrap();

        for row in 0..3 {
            for column in 0..3 {
                let product = (0..3)
                    .map(|index| matrix[row + index * 3] * inverse[index + column * 3])
                    .sum::<f64>();
                let expected = if row == column { 1.0 } else { 0.0 };
                assert!((product - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn matrix_inverse_reports_singular_matrix() {
        let matrix = [1.0, 2.0, 2.0, 4.0];

        let error = matrix_inverse(&matrix, 2).unwrap_err();

        assert_eq!(error, MatrixInverseError::Singular { pivot: 2 });
    }
}
