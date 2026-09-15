# cauchy-ode

Write equations naturally in Rust, with symbolic Jacobians and adaptive stiff
solvers. `cauchy-ode` solves initial value problems for ordinary differential
equations using England, Lawson, and Rosenbrock methods implemented in Fortran.

Start with [`Solver`] for a single solve, or construct an [`OdeSystem`] to
reuse parsed equations and declare their structure. [`MathExpr`] provides
standalone expression evaluation and symbolic differentiation.

## Installation and native dependencies

Add the crate to your application:

```toml
[dependencies]
cauchy-ode = "0.1"
```

The Rust import name is `cauchy_ode`. Building and running require a native
Fortran toolchain and numerical libraries:

| Platform | Native requirements |
| --- | --- |
| Linux | GNU Fortran, a C linker, `ar`, and BLAS/LAPACK development libraries |
| macOS | GNU Fortran and a C toolchain; Accelerate is supplied by macOS |
| Windows x86_64 MSVC | Intel Fortran (`ifx`), oneMKL, and Visual Studio C++ build tools |

See the [build guide](https://github.com/drnikolaev/Cauchy/blob/main/BUILDING.md)
for installation commands and runtime setup. Hosted documentation skips native
compilation; that does not remove these requirements for applications.

## Solve a scalar equation

Solve `x' = t + sqrt(x)` with `x(1) = 1` up to `t = 2`:

```rust
use cauchy_ode::Solver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let solution = Solver::default().solve("t+sqrt(x)", 1.0, 1.0, 2.0)?;
    let final_x = solution.states.last().unwrap()[0];
    assert!((final_x - 4.0).abs() < 1e-6);
    Ok(())
}

```

[`Solver::solve`] accepts `t` and either `x` or `x0` for the scalar state.
[`Solver::solve_system`] uses `t` and zero-based state names `x0`, `x1`, …:

```rust
use cauchy_ode::Solver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let solution = Solver::default().solve_system(
        &["x1", "-x0"],
        0.0,
        &[1.0, 0.0],
        std::f64::consts::TAU,
    )?;
    assert!((solution.states.last().unwrap()[0] - 1.0).abs() < 1e-6);
    Ok(())
}
```

## Named variables, parameters, and stiff systems

Construct a system explicitly to choose variable names and bind parameters.
For example, this equation has the exact solution `x(t) = cos(t)` but a fast
decaying mode with rate 1000:

```rust
use cauchy_ode::{Method, OdeSystem, Solver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system = OdeSystem::general_with_parameters(
        "t", &["x"], &["-rate*(x-cos(t))-sin(t)"], &[("rate", 1000.0)],
    )?;
    let solver = Solver {
        method: Method::Rosenbrock,
        tolerance: 1e-10,
        ..Solver::default()
    };
    let solution = solver.solve_problem(&system, 0.0, &[1.0], 1.0)?;
    assert!((solution.states.last().unwrap()[0] - 1.0_f64.cos()).abs() < 1e-6);
    Ok(())
}
```

[`OdeSystem`] uses exactly the names you declare, with no implicit aliases.
Parameter bindings are finite constants substituted before symbolic
differentiation. The solver prepares the Jacobian and partial time derivative
automatically when the selected method requires them.

## System types and methods

| Type | Equation | Constructor | Compatible methods |
| --- | --- | --- | --- |
| 1: General | `x' = f(t,x)` | [`OdeSystem::general`] | England, Lawson, Rosenbrock |
| 2: Autonomous | `x' = f(x)` | [`OdeSystem::autonomous`] | England, Lawson, Rosenbrock, RosenbrockAutonomous |
| 3: Linear | `x' = A(t)x + phi(t)` | [`OdeSystem::linear`] | England, Lawson, Rosenbrock, LawsonLinear |
| 4: Constant linear part | `x' = Bx + u(t,x)` | [`OdeSystem::split`] | England, Lawson, Rosenbrock, LawsonSplit |

Each constructor also has a `_with_parameters` variant. `new` and
`new_with_parameters` remain compatibility aliases for the general constructors.
Matrix entries use column-major order: `column * dimension + row`.
Linear forcing depends only on time; the split matrix is constant.

England is the default explicit method. Rosenbrock is a useful starting point
for nonlinear stiff systems; the specialized Lawson variants take advantage of
linear or split structure. Compatibility does not guarantee efficiency on every
problem. See [`Method::supports`] and the
[stiff-system limitations](https://github.com/drnikolaev/Cauchy#stiff-system-regression-tests).

## Expressions

The [`math_expr`] module contains the complete syntax reference, including all
operators and functions. Use explicit multiplication (`2*x`), `^` for powers,
and parentheses for grouping. `log(x)` is the natural logarithm. Trigonometric
functions use radians. There are no built-in `pi` or `e` constants; bind them
as parameters when needed.

`iif(check, negative, otherwise)` evaluates its second argument when the check
is negative, and its third argument otherwise. Piecewise differentiation does
not guarantee smoothness at a switching point or provide event detection.

## Results, accuracy, and errors

[`Solution`] stores the initial point and every accepted adaptive step,
including the requested endpoint. `states[k]` corresponds to `times[k]`.
`recommended_steps[k]` is the signed recommended next step (the initial step
at index zero); actual accepted steps are differences between adjacent times.
An end time earlier than the start time integrates backward.

[`Solver`] exposes `initial_step`, `min_step`, `max_step`, `tolerance`,
`relative_threshold`, and `max_steps`. Step-size settings are positive
magnitudes. Tolerance controls an estimated local error, not accumulated global
error. The relative/absolute criterion uses the maximum state amplitude and
the `relative_threshold` setting. The last step may be shorter than `min_step`.

Solves return [`SolverError`] for invalid inputs, incompatible methods, parse
or evaluation failures, non-finite required values, and integration failures.
Standalone [`MathExpr::evaluate`] can return NaN or infinity for domain errors;
solver callbacks check required values for finiteness.

## Examples and license

The [example guide](https://github.com/drnikolaev/Cauchy/blob/main/EXAMPLES.md)
includes a console demo, seven interactive attractor viewers, and comparisons
against exact solutions for all four system types. Clone the repository to run
these examples with `cargo run --release --example <name>`.

Distributed under the
[Boost Software License, Version 1.0](https://www.boost.org/LICENSE_1_0.txt).
