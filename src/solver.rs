//! Adaptive ODE integration with the Fortran England solver.

use crate::math_expr::{MathExpr, ParseError};
use std::collections::HashMap;
use std::ffi::{c_double, c_int, c_void};
use std::fmt::{Display, Formatter};
use std::panic::{AssertUnwindSafe, catch_unwind};

type RhsCallback = unsafe extern "C" fn(
    *const c_int,
    *const c_double,
    *const c_double,
    *mut c_double,
    *mut c_void,
    *mut c_int,
);

unsafe extern "C" {
    fn cauchy_sengl(
        callback: RhsCallback,
        context: *mut c_void,
        dimension: *const c_int,
        min_step: *const c_double,
        max_step: *const c_double,
        tolerance: *const c_double,
        relative_threshold: *const c_double,
        state: *mut c_double,
        time: *mut c_double,
        step: *mut c_double,
        work: *mut c_double,
        info: *mut c_int,
    );
}

/// Integration controls. Step sizes are positive magnitudes; the integration
/// direction is determined by the start and end times.
#[derive(Clone, Debug)]
pub struct Solver {
    pub initial_step: f64,
    pub min_step: f64,
    pub max_step: f64,
    /// SENGL's local error tolerance (EPS), not a global error bound.
    pub tolerance: f64,
    /// SENGL uses relative error when max(abs(state)) >= this threshold,
    /// and absolute error below it (P).
    pub relative_threshold: f64,
    /// Maximum number of accepted steps, excluding the initial point.
    pub max_steps: usize,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            initial_step: 0.01,
            min_step: 1e-12,
            max_step: 0.1,
            tolerance: 1e-9,
            relative_threshold: 1.0,
            max_steps: 100_000,
        }
    }
}

/// Initial point and every accepted adaptive step, including the endpoint.
/// `states[i]` is the state vector at `times[i]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    pub times: Vec<f64>,
    pub states: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SolverError {
    InvalidInput(&'static str),
    Parse { equation: usize, source: ParseError },
    Evaluation { equation: usize, message: String },
    NonFiniteDerivative { equation: usize, time: f64 },
    NonFiniteState { time: f64 },
    StepSizeTooSmall { time: f64 },
    StepLimitExceeded { time: f64 },
    NoProgress { time: f64 },
    CallbackPanicked,
    FortranFailure { info: c_int },
}

impl Display for SolverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => f.write_str(message),
            Self::Parse { equation, source } => write!(f, "equation {equation}: {source}"),
            Self::Evaluation { equation, message } => {
                write!(f, "equation {equation}: {message}")
            }
            Self::NonFiniteDerivative { equation, time } => {
                write!(
                    f,
                    "equation {equation} has a non-finite derivative at t={time}"
                )
            }
            Self::NonFiniteState { time } => write!(f, "non-finite time or state at t={time}"),
            Self::StepSizeTooSmall { time } => {
                write!(
                    f,
                    "SENGL cannot meet the tolerance at the minimum step at t={time}"
                )
            }
            Self::StepLimitExceeded { time } => write!(f, "step limit reached at t={time}"),
            Self::NoProgress { time } => {
                write!(f, "step cannot advance floating-point time at t={time}")
            }
            Self::CallbackPanicked => f.write_str("expression evaluation panicked"),
            Self::FortranFailure { info } => write!(f, "SENGL failed with IERR={info}"),
        }
    }
}

impl std::error::Error for SolverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl Solver {
    /// Solves `x' = rhs(t, x)` with `x(start_time) = initial_x`.
    /// The expression may use `t`, `x` (also `x0`), and MathExpr functions.
    ///
    /// ```
    /// use cauchy::Solver;
    /// let solution = Solver::default().solve("x", 0.0, 1.0, 1.0)?;
    /// let x = solution.states.last().unwrap()[0];
    /// assert!((x - std::f64::consts::E).abs() < 1e-7);
    /// # Ok::<(), cauchy::SolverError>(())
    /// ```
    pub fn solve(
        &self,
        rhs: &str,
        start_time: f64,
        initial_x: f64,
        end_time: f64,
    ) -> Result<Solution, SolverError> {
        self.solve_system(&[rhs], start_time, &[initial_x], end_time)
    }

    /// Solves a system with one RHS string per state component. Variables
    /// `x0`, `x1`, ... refer to zero-based components and `t` to time.
    /// `x` is an additional alias only for one-dimensional systems.
    ///
    /// Supports forward and backward integration. The final step may be shorter
    /// than `min_step` to land on `end_time`. A zero-length interval returns
    /// the initial point after validating inputs and parsing the expressions.
    pub fn solve_system(
        &self,
        rhs: &[&str],
        start_time: f64,
        initial_state: &[f64],
        end_time: f64,
    ) -> Result<Solution, SolverError> {
        self.validate(rhs, start_time, initial_state, end_time)?;
        // SENGL computes indices up to 7*M using the BLAS integer ABI.
        let work_len = initial_state
            .len()
            .checked_mul(7)
            .filter(|&len| c_int::try_from(len).is_ok())
            .ok_or(SolverError::InvalidInput(
                "system is too large for the Fortran integer ABI",
            ))?;
        let dimension = initial_state.len() as c_int;
        let expressions = rhs
            .iter()
            .enumerate()
            .map(|(equation, source)| {
                MathExpr::<f64>::parse(source)
                    .map_err(|source| SolverError::Parse { equation, source })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut context = CallbackContext {
            expressions,
            names: (0..initial_state.len()).map(|i| format!("x{i}")).collect(),
            error: None,
        };
        let mut solution = Solution {
            times: vec![start_time],
            states: vec![initial_state.to_vec()],
        };
        let mut time = start_time;
        let mut state = initial_state.to_vec();
        let mut work = vec![0.0; work_len];
        let direction = (end_time - start_time).signum();
        let mut step = direction * self.initial_step;
        let mut steps = 0;

        while direction * (end_time - time) > 0.0 {
            if steps == self.max_steps {
                return Err(SolverError::StepLimitExceeded { time });
            }
            let remaining = (end_time - time).abs();
            step = direction
                * step
                    .abs()
                    .clamp(self.min_step, self.max_step)
                    .min(remaining);
            let min_step = self.min_step.min(remaining);
            if time + step == time {
                return Err(SolverError::NoProgress { time });
            }
            let previous_time = time;
            let mut info = 0;

            // SAFETY: SENGL calls the callback synchronously and retains no
            // pointers. Context and all buffers live for the complete call.
            // X has M elements and R has 7*M elements, with indices fitting
            // c_int. All scalars are passed by reference except the context
            // and function pointer, matching the Fortran BIND(C) interface.
            unsafe {
                cauchy_sengl(
                    evaluate_rhs,
                    (&mut context as *mut CallbackContext<'_>).cast(),
                    &dimension,
                    &min_step,
                    &self.max_step,
                    &self.tolerance,
                    &self.relative_threshold,
                    state.as_mut_ptr(),
                    &mut time,
                    &mut step,
                    work.as_mut_ptr(),
                    &mut info,
                );
            }
            if let Some(error) = context.error.take() {
                return Err(error);
            }
            match info {
                0 => {}
                65 => return Err(SolverError::StepSizeTooSmall { time }),
                info => return Err(SolverError::FortranFailure { info }),
            }
            if !time.is_finite() || !state.iter().all(|x| x.is_finite()) {
                return Err(SolverError::NonFiniteState { time });
            }
            if direction * (time - previous_time) <= 0.0 {
                return Err(SolverError::NoProgress { time });
            }
            solution.times.push(time);
            solution.states.push(state.clone());
            steps += 1;
        }
        Ok(solution)
    }

    fn validate(
        &self,
        rhs: &[&str],
        start_time: f64,
        initial_state: &[f64],
        end_time: f64,
    ) -> Result<(), SolverError> {
        if rhs.is_empty() || rhs.len() != initial_state.len() {
            return Err(SolverError::InvalidInput(
                "provide one RHS per state component and at least one component",
            ));
        }
        if !start_time.is_finite() || !end_time.is_finite() || !(end_time - start_time).is_finite()
        {
            return Err(SolverError::InvalidInput(
                "times and their difference must be finite",
            ));
        }
        if !initial_state.iter().all(|x| x.is_finite()) {
            return Err(SolverError::InvalidInput("initial state must be finite"));
        }
        if ![
            self.initial_step,
            self.min_step,
            self.max_step,
            self.tolerance,
            self.relative_threshold,
        ]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
        {
            return Err(SolverError::InvalidInput(
                "step sizes, tolerance, and relative threshold must be finite and positive",
            ));
        }
        if self.min_step > self.initial_step || self.initial_step > self.max_step {
            return Err(SolverError::InvalidInput(
                "require min_step <= initial_step <= max_step",
            ));
        }
        if self.max_steps == 0 {
            return Err(SolverError::InvalidInput("max_steps must be positive"));
        }
        Ok(())
    }
}

struct CallbackContext<'a> {
    expressions: Vec<MathExpr<'a, f64>>,
    names: Vec<String>,
    error: Option<SolverError>,
}

impl CallbackContext<'_> {
    fn evaluate(
        &self,
        time: f64,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), SolverError> {
        if !time.is_finite() || !state.iter().all(|x| x.is_finite()) {
            return Err(SolverError::NonFiniteState { time });
        }
        let mut variables: HashMap<&str, f64> = self
            .names
            .iter()
            .zip(state)
            .map(|(name, &value)| (name.as_str(), value))
            .collect();
        variables.insert("t", time);
        if state.len() == 1 {
            variables.insert("x", state[0]);
        }
        for (equation, (expression, output)) in self.expressions.iter().zip(derivative).enumerate()
        {
            let value =
                expression
                    .evaluate(&variables)
                    .map_err(|error| SolverError::Evaluation {
                        equation,
                        message: format!("{error:?}"),
                    })?;
            if !value.is_finite() {
                return Err(SolverError::NonFiniteDerivative { equation, time });
            }
            *output = value;
        }
        Ok(())
    }
}

// Only SENGL invokes this private callback. Its input and output buffers are
// non-overlapping and contain M doubles; scalars and context are valid for the
// duration of each call. No Rust unwind is allowed to cross the FFI boundary.
unsafe extern "C" fn evaluate_rhs(
    dimension: *const c_int,
    time: *const c_double,
    state: *const c_double,
    derivative: *mut c_double,
    context: *mut c_void,
    info: *mut c_int,
) {
    let context = unsafe { &mut *context.cast::<CallbackContext<'_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| {
        let dimension = unsafe { *dimension } as usize;
        let time = unsafe { *time };
        let state = unsafe { std::slice::from_raw_parts(state, dimension) };
        let derivative = unsafe { std::slice::from_raw_parts_mut(derivative, dimension) };
        context.evaluate(time, state, derivative)
    }));
    let error = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error),
        Err(_) => Some(SolverError::CallbackPanicked),
    };
    unsafe { *info = if error.is_some() { -1 } else { 0 } };
    context.error = error;
}
