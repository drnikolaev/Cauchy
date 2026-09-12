//! Parsed ODEs, independent of the algorithm used to integrate them.
use crate::{MathExpr, SolverError};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemKind {
    General,
    Autonomous,
    /// x' = A(t)x + phi(t), as implemented by SLOUI.
    TimeLinear,
    /// x' = Bx + u(t,x), with constant B.
    ConstantLinear,
}

/// A parsed vector field with named state variables. Matrices are column-major:
/// element (row, column) has index `column * dimension + row`.
/// Names are explicit: there are no reserved `x`, `x0`, or `t` aliases here.
#[derive(Clone, Debug)]
pub struct OdeSystem<'a> {
    pub(crate) time: Option<&'a str>,
    pub(crate) names: Vec<&'a str>,
    pub(crate) rhs: Vec<MathExpr<'a, f64>>,
    pub(crate) matrix: Vec<MathExpr<'a, f64>>,
    pub(crate) forcing: Vec<MathExpr<'a, f64>>,
    pub(crate) constant_matrix: Vec<f64>,
    kind: SystemKind,
}

impl<'a> OdeSystem<'a> {
    /// Creates x' = F(t,x). Names and expressions are borrowed; parsing happens
    /// only here, so a system can be reused for multiple solves.
    pub fn new(time: &'a str, names: &[&'a str], rhs: &[&'a str]) -> Result<Self, SolverError> {
        Self::new_with_parameters(time, names, rhs, &[])
    }

    /// Creates a general system with named, finite constant parameters.
    /// Bindings are substituted structurally before variable validation and
    /// differentiation. Parameter names must be valid identifiers, unique, and
    /// distinct from state/time names. Unused bindings are allowed. Rebuild the
    /// system to change parameter values; no parameter map is needed at runtime.
    ///
    /// ```
    /// use cauchy_ode::{OdeSystem, Solver};
    /// let system = OdeSystem::new_with_parameters(
    ///     "clock", &["position"], &["-rate*position"], &[("rate", 2.0)],
    /// )?;
    /// let trajectory = Solver::default().solve_problem(&system, 0.0, &[1.0], 1.0)?;
    /// assert!((trajectory.states.last().unwrap()[0] - (-2.0_f64).exp()).abs() < 1e-6);
    /// # Ok::<(), cauchy_ode::SolverError>(())
    /// ```
    pub fn new_with_parameters(
        time: &'a str,
        names: &[&'a str],
        rhs: &[&'a str],
        parameters: &[(&str, f64)],
    ) -> Result<Self, SolverError> {
        Self::parse(Some(time), names, rhs, SystemKind::General, parameters)
    }

    /// Creates x' = F(x). Time-dependent expressions are rejected.
    pub fn autonomous(names: &[&'a str], rhs: &[&'a str]) -> Result<Self, SolverError> {
        Self::autonomous_with_parameters(names, rhs, &[])
    }

    /// Creates an autonomous system with constant bindings; see
    /// [`Self::new_with_parameters`] for the binding rules.
    pub fn autonomous_with_parameters(
        names: &[&'a str],
        rhs: &[&'a str],
        parameters: &[(&str, f64)],
    ) -> Result<Self, SolverError> {
        Self::parse(None, names, rhs, SystemKind::Autonomous, parameters)
    }

    /// Creates x' = A(t)x + phi(t). A and phi may depend only on the named time
    /// variable. For state-dependent forcing use `new` with the complete RHS,
    /// or `split` when the linear matrix is constant.
    pub fn linear(
        time: &'a str,
        names: &[&'a str],
        matrix: &[&'a str],
        forcing: &[&'a str],
    ) -> Result<Self, SolverError> {
        Self::linear_with_parameters(time, names, matrix, forcing, &[])
    }

    /// Binds constants in both A(t) and phi(t). After binding, only the named
    /// time variable may remain in either expression collection.
    pub fn linear_with_parameters(
        time: &'a str,
        names: &[&'a str],
        matrix: &[&'a str],
        forcing: &[&'a str],
        parameters: &[(&str, f64)],
    ) -> Result<Self, SolverError> {
        let mut system = Self::parse(
            Some(time),
            names,
            forcing,
            SystemKind::TimeLinear,
            parameters,
        )?;
        Self::check_matrix_size(names.len(), matrix.len())?;
        system.forcing = system.rhs.clone();
        system.matrix = Self::parse_expressions(matrix, parameters)?;
        for (equation, expression) in system.matrix.iter().chain(&system.forcing).enumerate() {
            Self::check_variables(expression, &[time], equation)?;
        }
        system.rhs = system.assemble_rhs(&system.matrix, &system.forcing);
        Ok(system)
    }

    /// Creates x' = Bx + u(t,x), with a finite constant column-major matrix B.
    pub fn split(
        time: &'a str,
        names: &[&'a str],
        matrix: &[f64],
        remainder: &[&'a str],
    ) -> Result<Self, SolverError> {
        let mut system = Self::parse(
            Some(time),
            names,
            remainder,
            SystemKind::ConstantLinear,
            &[],
        )?;
        Self::check_matrix_size(names.len(), matrix.len())?;
        if !matrix.iter().all(|v| v.is_finite()) {
            return Err(SolverError::InvalidInput("constant matrix must be finite"));
        }
        system.constant_matrix = matrix.to_vec();
        system.forcing = system.rhs.clone();
        system.matrix = matrix.iter().map(|&v| MathExpr::new_const(v)).collect();
        system.rhs = system.assemble_rhs(&system.matrix, &system.forcing);
        Ok(system)
    }

    /// Creates Bx + u(t,x) with parameters in both B and u. Unlike [`Self::split`],
    /// B is supplied as column-major expression strings (e.g. "-rate"). Each
    /// matrix entry must evaluate to a finite constant after parameter binding;
    /// time/state-dependent matrices are rejected.
    pub fn split_with_parameters(
        time: &'a str,
        names: &[&'a str],
        matrix: &[&'a str],
        remainder: &[&'a str],
        parameters: &[(&str, f64)],
    ) -> Result<Self, SolverError> {
        let mut system = Self::parse(
            Some(time),
            names,
            remainder,
            SystemKind::ConstantLinear,
            parameters,
        )?;
        Self::check_matrix_size(names.len(), matrix.len())?;
        let entries = Self::parse_expressions(matrix, parameters)?;
        let args = HashMap::new();
        for (equation, entry) in entries.iter().enumerate() {
            Self::check_variables(entry, &[], equation)?;
            let value = entry
                .evaluate(&args)
                .map_err(|error| SolverError::Evaluation {
                    equation,
                    message: format!("{error:?}"),
                })?;
            if !value.is_finite() {
                return Err(SolverError::InvalidInput("constant matrix must be finite"));
            }
            system.constant_matrix.push(value);
        }
        system.forcing = system.rhs.clone();
        system.matrix = system
            .constant_matrix
            .iter()
            .map(|&v| MathExpr::new_const(v))
            .collect();
        system.rhs = system.assemble_rhs(&system.matrix, &system.forcing);
        Ok(system)
    }

    pub fn kind(&self) -> SystemKind {
        self.kind
    }
    pub fn dimension(&self) -> usize {
        self.names.len()
    }
    pub fn variables(&self) -> &[&'a str] {
        &self.names
    }
    pub fn expressions(&self) -> &[MathExpr<'a, f64>] {
        &self.rhs
    }

    fn parse(
        time: Option<&'a str>,
        names: &[&'a str],
        rhs: &[&'a str],
        kind: SystemKind,
        parameters: &[(&str, f64)],
    ) -> Result<Self, SolverError> {
        let mut used: HashSet<_> = names.iter().copied().chain(time).collect();
        for &(name, value) in parameters {
            if !Self::valid_identifier(name) || !used.insert(name) {
                return Err(SolverError::InvalidInput(
                    "parameter names must be valid, unique, and distinct from time/state names",
                ));
            }
            if !value.is_finite() {
                return Err(SolverError::InvalidInput("parameter values must be finite"));
            }
        }
        Self::from_expressions(time, names, Self::parse_expressions(rhs, parameters)?, kind)
    }

    // The convenience solver can normalize aliases in its parsed trees before
    // entering the same strict name-validation path as the public constructors.
    pub(crate) fn from_expressions(
        time: Option<&'a str>,
        names: &[&'a str],
        expressions: Vec<MathExpr<'a, f64>>,
        kind: SystemKind,
    ) -> Result<Self, SolverError> {
        if names.is_empty() || names.len() != expressions.len() {
            return Err(SolverError::InvalidInput(
                "provide one RHS per named state component",
            ));
        }
        let mut unique = HashSet::new();
        for name in names.iter().copied().chain(time) {
            if !Self::valid_identifier(name) || !unique.insert(name) {
                return Err(SolverError::InvalidInput(
                    "time and state names must be distinct identifiers, not numeric literals",
                ));
            }
        }
        let allowed: Vec<_> = unique.into_iter().collect();
        for (equation, expression) in expressions.iter().enumerate() {
            Self::check_variables(expression, &allowed, equation)?;
        }
        Ok(Self {
            time,
            names: names.to_vec(),
            rhs: expressions,
            matrix: vec![],
            forcing: vec![],
            constant_matrix: vec![],
            kind,
        })
    }

    fn valid_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        matches!(chars.next(), Some(c) if c.is_alphabetic() || c == '_')
            && chars.all(|c| c.is_alphanumeric() || c == '_')
            && name.parse::<f64>().is_err()
    }

    fn parse_expressions(
        sources: &[&'a str],
        parameters: &[(&str, f64)],
    ) -> Result<Vec<MathExpr<'a, f64>>, SolverError> {
        sources
            .iter()
            .enumerate()
            .map(|(equation, source)| {
                let mut expression = MathExpr::parse(source)
                    .map_err(|source| SolverError::Parse { equation, source })?;
                for &(name, value) in parameters {
                    expression = expression.substitute(name, &MathExpr::new_const(value));
                }
                Ok(expression)
            })
            .collect()
    }

    fn check_variables(
        expression: &MathExpr<'_, f64>,
        allowed: &[&str],
        equation: usize,
    ) -> Result<(), SolverError> {
        if let Some(name) = expression
            .variables()
            .into_iter()
            .filter(|name| !allowed.contains(name))
            .min()
        {
            return Err(SolverError::Evaluation {
                equation,
                message: format!("variable '{name}' is not allowed in this system expression"),
            });
        }
        Ok(())
    }

    fn check_matrix_size(dimension: usize, len: usize) -> Result<(), SolverError> {
        if dimension.checked_mul(dimension) != Some(len) {
            return Err(SolverError::InvalidInput(
                "matrix must contain dimension * dimension elements",
            ));
        }
        Ok(())
    }

    fn assemble_rhs(
        &self,
        matrix: &[MathExpr<'a, f64>],
        forcing: &[MathExpr<'a, f64>],
    ) -> Vec<MathExpr<'a, f64>> {
        forcing
            .iter()
            .enumerate()
            .map(|(row, force)| {
                self.names
                    .iter()
                    .enumerate()
                    .fold(force.clone(), |rhs, (column, name)| {
                        MathExpr::new_add(
                            rhs,
                            MathExpr::new_multiply(
                                matrix[column * self.dimension() + row].clone(),
                                MathExpr::new_var(name),
                            ),
                        )
                    })
            })
            .collect()
    }
}
