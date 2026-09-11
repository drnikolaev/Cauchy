# Cauchy
Cauchy initial value problem solver

`Solver` integrates expressions with the adaptive England method in
`ftn/sengl.f`. Build with Cargo and a Fortran compiler (`gfortran` by default,
or set `FC`). Linking uses Accelerate on macOS and BLAS/LAPACK elsewhere.

```rust
use cauchy::Solver;

// x' = t + sqrt(x), x(1) = 1, integrate to t = 2.
let solution = Solver::default()
    .solve("t+sqrt(x)", 1.0, 1.0, 2.0)
    .expect("ODE integration failed");
let final_x = solution.states.last().unwrap()[0]; // approximately 4
```

`solve(rhs, start_time, initial_x, end_time)` accepts `t` and `x` in the RHS.
For systems, use one expression per component and zero-based variable names:

```rust
use cauchy::Solver;

// x0' = x1, x1' = -x0.
let solution = Solver::default().solve_system(
    &["x1", "-x0"], 0.0, &[1.0, 0.0], std::f64::consts::TAU,
).expect("ODE integration failed");
```

The result contains the initial point and every accepted adaptive step in
`times` and `states`, including the requested endpoint. Earlier end times
perform backward integration. Configure `initial_step`, `min_step`, `max_step`,
`tolerance`, `relative_threshold`, and `max_steps` on `Solver`; `tolerance` is
SENGL's per-step error tolerance, not a guarantee on the final error. The final
step may be shorter than `min_step` to reach the endpoint.

Expressions are parsed once. SENGL's C-compatible callback receives an opaque
Rust context and calls `MathExpr::evaluate` with each stage's time and state.
No shared callback state is used. Parse/evaluation errors, non-finite results,
minimum-step failures (`IERR=65`), and step limits return `SolverError`.
The Fortran interface is now `SENGL(F, CONTEXT, M, HMIN, HMAX, EPS, P, X, T,
H, R, IERR)`, exported as `cauchy_sengl`; `F(M,T,X,Z,CONTEXT,IERR)` sets zero
status on success. `CONTEXT` is passed by value and numerical arguments by
reference, following [Fortran's C interoperability conventions](https://gcc.gnu.org/onlinedocs/gfortran/Working-with-C-Pointers.html).
