// SPDX-License-Identifier: BSL-1.0
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE or https://www.boost.org/LICENSE_1_0.txt.

use cauchy_ode::fortran::matrix_exp;

#[test]
fn call_fortran_matrix_exp_from_rust() {
    let matrix = [
        0.0, 1.0, // column 1
        0.0, 0.0, // column 2
    ];

    let result = matrix_exp(&matrix, 2, 1e-14);

    assert_eq!(result, [1.0, 1.0, 0.0, 1.0]);
}
