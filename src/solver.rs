//! Adaptive integration with the Fortran England, Lawson and Rosenbrock solvers.

use crate::math_expr::{MathExpr, ParseError};
use crate::{OdeSystem, SystemKind};
use std::collections::HashMap;
use std::ffi::c_int;
use std::fmt::{Display, Formatter};
mod ffi;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Method {
    #[default]
    England,
    Lawson,
    LawsonLinear,
    LawsonSplit,
    Rosenbrock,
    RosenbrockAutonomous,
}

impl Method {
    pub fn supports(self, kind: SystemKind) -> bool {
        match self {
            Self::LawsonLinear => kind == SystemKind::TimeLinear,
            Self::LawsonSplit => kind == SystemKind::ConstantLinear,
            Self::RosenbrockAutonomous => kind == SystemKind::Autonomous,
            _ => true,
        }
    }

    fn workspace_len(self, dimension: usize) -> Result<usize, SolverError> {
        let (quadratic, linear) = match self {
            Self::England => (0, 7),
            Self::Lawson | Self::Rosenbrock => (6, 6),
            Self::LawsonLinear => (7, 5),
            Self::LawsonSplit => (5, 6),
            Self::RosenbrockAutonomous => (6, 4),
        };
        let len = dimension
            .checked_mul(dimension)
            .and_then(|square| square.checked_mul(quadratic))
            .and_then(|square| {
                dimension
                    .checked_mul(linear)
                    .and_then(|v| square.checked_add(v))
            })
            .filter(|&len| c_int::try_from(len).is_ok());
        // INV also allocates 64*M doubles for LAPACK work.
        if dimension
            .checked_mul(64)
            .is_none_or(|v| c_int::try_from(v).is_err())
        {
            return Err(SolverError::InvalidInput(
                "system is too large for the Fortran integer ABI",
            ));
        }
        len.ok_or(SolverError::InvalidInput(
            "system is too large for the Fortran integer ABI",
        ))
    }
}

/// Integration controls. Step sizes are positive magnitudes; the integration
/// direction is determined by the start and end times.
#[derive(Clone, Debug)]
pub struct Solver {
    pub method: Method,
    pub initial_step: f64,
    pub min_step: f64,
    pub max_step: f64,
    /// Local error tolerance (EPS), not a global error bound.
    pub tolerance: f64,
    /// Solvers use relative error when max(abs(state)) >= this threshold,
    /// and absolute error below it (P).
    pub relative_threshold: f64,
    /// Maximum number of accepted steps, excluding the initial point.
    pub max_steps: usize,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            method: Method::England,
            initial_step: 0.01,
            min_step: 1e-12,
            max_step: 0.1,
            tolerance: 1e-9,
            relative_threshold: 1.0,
            max_steps: 10_000_000,
        }
    }
}

/// Initial point and every accepted adaptive step, including the endpoint.
/// `states[i]` is the state vector at `times[i]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    pub times: Vec<f64>,
    pub states: Vec<Vec<f64>>,
    /// Signed recommended next step at each point (initial step at index zero).
    /// Actual accepted steps are successive differences in `times`.
    pub recommended_steps: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SolverError {
    InvalidInput(&'static str),
    IncompatibleMethod { method: Method, system: SystemKind },
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
            Self::IncompatibleMethod { method, system } => {
                write!(f, "{method:?} does not support {system:?} systems")
            }
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
                    "solver cannot meet the tolerance at the minimum step at t={time}"
                )
            }
            Self::StepLimitExceeded { time } => write!(f, "step limit reached at t={time}"),
            Self::NoProgress { time } => {
                write!(f, "step cannot advance floating-point time at t={time}")
            }
            Self::CallbackPanicked => f.write_str("expression evaluation panicked"),
            Self::FortranFailure { info } => write!(f, "Fortran solver failed with IERR={info}"),
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
    /// use cauchy_ode::Solver;
    /// let solution = Solver::default().solve("x", 0.0, 1.0, 1.0)?;
    /// let x = solution.states.last().unwrap()[0];
    /// assert!((x - std::f64::consts::E).abs() < 1e-7);
    /// # Ok::<(), cauchy_ode::SolverError>(())
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
        let names: Vec<_> = (0..rhs.len()).map(|i| format!("x{i}")).collect();
        let names: Vec<_> = names.iter().map(String::as_str).collect();
        let expressions = rhs
            .iter()
            .enumerate()
            .map(|(equation, source)| {
                let expression = MathExpr::parse(source)
                    .map_err(|source| SolverError::Parse { equation, source })?;
                // Only the convenience API defines x as an alias for scalar x0.
                // Normalize before building Jacobians, without a text round-trip.
                Ok(if names.len() == 1 {
                    expression.substitute("x", &MathExpr::new_var("x0"))
                } else {
                    expression
                })
            })
            .collect::<Result<Vec<_>, SolverError>>()?;
        let (time, kind) = if self.method == Method::RosenbrockAutonomous {
            (None, SystemKind::Autonomous)
        } else {
            (Some("t"), SystemKind::General)
        };
        let system = OdeSystem::from_expressions(time, &names, expressions, kind)?;
        self.solve_problem(&system, start_time, initial_state, end_time)
    }

    /// Integrates a reusable parsed system with this solver's selected method.
    /// Named variables and structured linear forms are specified by OdeSystem.
    pub fn solve_problem(
        &self,
        system: &OdeSystem<'_>,
        start_time: f64,
        initial_state: &[f64],
        end_time: f64,
    ) -> Result<Solution, SolverError> {
        self.validate(system.dimension(), start_time, initial_state, end_time)?;
        if !self.method.supports(system.kind()) {
            return Err(SolverError::IncompatibleMethod {
                method: self.method,
                system: system.kind(),
            });
        }
        let work_len = self.method.workspace_len(system.dimension())?;
        let mut context = CallbackContext::new(system, self.method);
        let direction = (end_time - start_time).signum();
        let mut solution = Solution {
            times: vec![start_time],
            states: vec![initial_state.to_vec()],
            recommended_steps: vec![direction * self.initial_step],
        };
        let mut time = start_time;
        let mut state = initial_state.to_vec();
        let mut work = vec![0.0; work_len];
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
            let info = ffi::step(
                self,
                &mut context,
                min_step,
                &mut state,
                &mut time,
                &mut step,
                &mut work,
            );
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
            solution.recommended_steps.push(step);
            steps += 1;
        }
        Ok(solution)
    }

    fn validate(
        &self,
        dimension: usize,
        start_time: f64,
        initial_state: &[f64],
        end_time: f64,
    ) -> Result<(), SolverError> {
        if dimension == 0 || dimension != initial_state.len() {
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

#[derive(Clone, Copy)]
enum CallbackKind {
    Rhs,
    Jacobian,
    TimeDerivative,
    Matrix,
    Forcing,
}

struct CallbackContext<'s, 'a> {
    system: &'s OdeSystem<'a>,
    jacobian: Vec<MathExpr<'a, f64>>,
    time_derivative: Vec<MathExpr<'a, f64>>,
    variables: HashMap<&'a str, f64>,
    current_time: f64,
    error: Option<SolverError>,
}

impl<'s, 'a> CallbackContext<'s, 'a> {
    fn new(system: &'s OdeSystem<'a>, method: Method) -> Self {
        let mut jacobian = Vec::new();
        if matches!(
            method,
            Method::Lawson | Method::Rosenbrock | Method::RosenbrockAutonomous
        ) {
            // Column-major: each column differentiates every RHS by one variable.
            for name in &system.names {
                for expression in &system.rhs {
                    jacobian.push(expression.derive(name).simplify());
                }
            }
        }
        let time_derivative = if method == Method::Rosenbrock {
            system
                .rhs
                .iter()
                .map(|expression| match system.time {
                    Some(time) => expression.derive(time).simplify(),
                    None => MathExpr::new_const(0.0),
                })
                .collect()
        } else {
            Vec::new()
        };
        let variables = system
            .names
            .iter()
            .copied()
            .chain(system.time)
            .map(|name| (name, 0.0))
            .collect();
        Self {
            system,
            jacobian,
            time_derivative,
            variables,
            current_time: 0.0,
            error: None,
        }
    }

    fn evaluate(
        &mut self,
        time: f64,
        state: &[f64],
        derivative: &mut [f64],
        kind: CallbackKind,
    ) -> Result<(), SolverError> {
        if !time.is_finite() || !state.iter().all(|x| x.is_finite()) {
            return Err(SolverError::NonFiniteState { time });
        }
        for (&name, &value) in self.system.names.iter().zip(state) {
            self.variables.insert(name, value);
        }
        if let Some(name) = self.system.time {
            self.variables.insert(name, time);
        }
        let expressions = match kind {
            CallbackKind::Rhs => &self.system.rhs,
            CallbackKind::Jacobian => &self.jacobian,
            CallbackKind::TimeDerivative => &self.time_derivative,
            CallbackKind::Matrix => &self.system.matrix,
            CallbackKind::Forcing => &self.system.forcing,
        };
        assert_eq!(expressions.len(), derivative.len());
        for (equation, (expression, output)) in expressions.iter().zip(derivative).enumerate() {
            let value =
                expression
                    .evaluate(&self.variables)
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
