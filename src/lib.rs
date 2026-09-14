// SPDX-License-Identifier: BSL-1.0
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE or https://www.boost.org/LICENSE_1_0.txt.

//! Adaptive ODE solvers with symbolic expressions and Fortran implementations.
//!
//! Distributed under the Boost Software License, Version 1.0.
//! See the accompanying `LICENSE` file or <https://www.boost.org/LICENSE_1_0.txt>.

pub mod fortran;
pub mod math_expr;
pub mod ode_system;
pub mod solver;

pub use math_expr::MathExpr;
pub use ode_system::{OdeSystem, SystemKind};
pub use solver::{Method, Solution, Solver, SolverError};
