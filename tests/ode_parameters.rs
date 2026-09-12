use cauchy_ode::{MathExpr, Method, OdeSystem, Solver, SolverError};
use std::collections::HashMap;

#[test]
fn named_parameter_decay_example() -> Result<(), SolverError> {
    // Bind rate once: position' = -2 * position, with time named clock.
    let system = OdeSystem::new_with_parameters(
        "clock",
        &["position"],
        &["-rate*position"],
        &[("rate", 2.0)],
    )?;

    // Arguments: system, initial time, initial state, final time.
    // Initial-state components follow the declared variable order.
    let trajectory = Solver::default().solve_problem(&system, 0.0, &[1.0], 1.0)?;

    assert_eq!(trajectory.times[0], 0.0);
    assert_eq!(trajectory.states[0], [1.0]);
    assert_eq!(trajectory.times.last(), Some(&1.0));
    assert_eq!(trajectory.times.len(), trajectory.states.len());
    assert!(trajectory.times.windows(2).all(|times| times[1] > times[0]));

    // For position(0)=1, the exact solution is position(clock)=exp(-2*clock).
    for (&clock, state) in trajectory.times.iter().zip(&trajectory.states) {
        assert_eq!(state.len(), 1);
        let position = state[0];
        let expected = (-2.0 * clock).exp();
        assert!(
            (position - expected).abs() < 1e-7,
            "clock={clock}: position={position}, expected={expected}"
        );
    }
    Ok(())
}

fn integrate(
    method: Method,
    system: &OdeSystem<'_>,
    initial: &[f64],
    exact: impl Fn(f64) -> Vec<f64>,
) {
    let solution = Solver {
        method,
        ..Solver::default()
    }
    .solve_problem(system, 0.0, initial, 1.0)
    .unwrap();
    assert_eq!(solution.states[0], initial);
    assert_eq!(solution.times.last(), Some(&1.0));
    for (&time, state) in solution.times.iter().zip(&solution.states) {
        let expected_state = exact(time);
        assert_eq!(state.len(), expected_state.len());
        for (&actual, expected) in state.iter().zip(expected_state) {
            assert!(
                (actual - expected).abs() < 1e-6,
                "{method:?}, t={time}: {actual} != {expected}"
            );
        }
    }
}

#[test]
fn explicit_systems_do_not_alias_undeclared_x() {
    for result in [
        OdeSystem::new("clock", &["x0"], &["x"]),
        OdeSystem::autonomous(&["x0"], &["x"]),
        OdeSystem::linear("clock", &["x0"], &["0"], &["x"]),
        OdeSystem::split("clock", &["x0"], &[0.0], &["x"]),
    ] {
        assert!(
            matches!(result, Err(SolverError::Evaluation { message, .. })
            if message.contains("'x'"))
        );
    }
}

#[test]
fn time_can_be_named_x_without_becoming_a_state_alias() {
    let system = OdeSystem::new("x", &["x0"], &["2*x"]).unwrap();
    for method in [Method::England, Method::Lawson, Method::Rosenbrock] {
        integrate(method, &system, &[0.0], |t| vec![t * t]);
    }
}

#[test]
fn t_can_be_a_state_when_time_has_another_name() {
    let system = OdeSystem::new("clock", &["t", "velocity"], &["velocity", "-t"]).unwrap();
    for method in [Method::England, Method::Lawson, Method::Rosenbrock] {
        integrate(method, &system, &[1.0, 0.0], |t| vec![t.cos(), -t.sin()]);
    }
}

#[test]
fn convenience_scalar_aliases_still_share_one_derivative() {
    for method in [
        Method::England,
        Method::Lawson,
        Method::Rosenbrock,
        Method::RosenbrockAutonomous,
    ] {
        let solver = Solver {
            method,
            ..Solver::default()
        };
        for solution in [
            solver.solve("-x*x0", 0.0, 1.0, 1.0),
            solver.solve_system(&["-x*x0"], 0.0, &[1.0], 1.0),
        ] {
            let solution = solution.unwrap();
            for (&t, state) in solution.times.iter().zip(&solution.states) {
                assert!((state[0] - 1.0 / (1.0 + t)).abs() < 1e-6, "{method:?}");
            }
        }
        assert!(
            solver
                .solve_system(&["x", "x0"], 0.0, &[1.0, 1.0], 1.0)
                .is_err()
        );
    }
}

#[test]
fn general_parameters_are_bound_before_state_and_time_derivatives() {
    let system = OdeSystem::new_with_parameters(
        "clock",
        &["position"],
        &["-rate*(position-clock^2)+2*clock"],
        &[("rate", 1000.0)],
    )
    .unwrap();
    let args = HashMap::from([("clock", 0.5), ("position", 0.25)]);
    let rhs = &system.expressions()[0];
    assert!(!rhs.variables().contains("rate"));
    assert_eq!(
        rhs.derive("position").simplify().evaluate(&args),
        Ok(-1000.0)
    );
    assert_eq!(rhs.derive("clock").simplify().evaluate(&args), Ok(1002.0));
    for method in [Method::Lawson, Method::Rosenbrock] {
        integrate(method, &system, &[0.0], |t| vec![t * t]);
    }
}

#[test]
fn x_can_be_an_explicit_parameter_for_state_x0() {
    let system =
        OdeSystem::new_with_parameters("clock", &["x0"], &["-x*x0"], &[("x", 2.0)]).unwrap();
    assert_eq!(
        system.expressions()[0].derive("x0").simplify(),
        MathExpr::new_const(-2.0)
    );
    integrate(Method::Rosenbrock, &system, &[1.0], |t| {
        vec![(-2.0 * t).exp()]
    });
}

#[test]
fn autonomous_parameters_do_not_reserve_t() {
    let system =
        OdeSystem::autonomous_with_parameters(&["amount"], &["-t*amount"], &[("t", 2.0)]).unwrap();
    integrate(Method::RosenbrockAutonomous, &system, &[1.0], |t| {
        vec![(-2.0 * t).exp()]
    });
    assert!(OdeSystem::autonomous(&["amount"], &["-t*amount"]).is_err());
}

#[test]
fn linear_parameters_bind_matrix_and_forcing() {
    let system = OdeSystem::linear_with_parameters(
        "clock",
        &["amount"],
        &["-rate-clock"],
        &["(rate+clock-1)*exp(-clock)"],
        &[("rate", 2.0)],
    )
    .unwrap();
    for method in [
        Method::LawsonLinear,
        Method::Lawson,
        Method::Rosenbrock,
        Method::England,
    ] {
        integrate(method, &system, &[1.0], |t| vec![(-t).exp()]);
    }
}

#[test]
fn split_parameters_bind_constant_matrix_expressions_and_nonlinear_remainder() {
    let system = OdeSystem::split_with_parameters(
        "clock",
        &["amount"],
        &["-(rate+offset)"],
        &["-loss*amount^2"],
        &[("rate", 1.5), ("offset", 0.5), ("loss", 1.0)],
    )
    .unwrap();
    for method in [
        Method::LawsonSplit,
        Method::Lawson,
        Method::Rosenbrock,
        Method::England,
    ] {
        integrate(method, &system, &[1.0], |t| {
            let e = (-2.0 * t).exp();
            vec![2.0 * e / (3.0 - e)]
        });
    }
}

#[test]
fn binding_is_structural_not_textual() {
    // Binding k must not alter the state name kappa or the function name sin.
    let system = OdeSystem::autonomous_with_parameters(
        &["kappa"],
        &["k*kappa+sin(kappa)+sin"],
        &[("k", 2.0), ("sin", 3.0)],
    )
    .unwrap();
    assert_eq!(
        system.expressions()[0].variables(),
        ["kappa"].into_iter().collect()
    );
    let value = system.expressions()[0]
        .evaluate(&HashMap::from([("kappa", 0.5)]))
        .unwrap();
    assert_eq!(value, 4.0 + 0.5_f64.sin());
}

#[test]
fn parameter_values_are_copied_and_systems_are_independent() {
    let mut bindings = vec![("rate", 2.0), ("unused", 5.0)];
    let first = OdeSystem::autonomous_with_parameters(&["q"], &["-rate*q"], &bindings).unwrap();
    bindings[0].1 = 3.0;
    let second = OdeSystem::autonomous_with_parameters(&["q"], &["-rate*q"], &bindings).unwrap();
    drop(bindings);
    integrate(Method::RosenbrockAutonomous, &first, &[1.0], |t| {
        vec![(-2.0 * t).exp()]
    });
    integrate(Method::RosenbrockAutonomous, &second, &[1.0], |t| {
        vec![(-3.0 * t).exp()]
    });
}

#[test]
fn rejects_invalid_duplicate_colliding_and_nonfinite_bindings() {
    for bindings in [
        vec![("q", 1.0)],
        vec![("clock", 1.0)],
        vec![("rate", 1.0), ("rate", 2.0)],
        vec![("", 1.0)],
        vec![("1rate", 1.0)],
        vec![("a-b", 1.0)],
        vec![("NaN", 1.0)],
        vec![("rate", f64::NAN)],
        vec![("rate", f64::INFINITY)],
        vec![("rate", f64::NEG_INFINITY)],
    ] {
        assert!(
            matches!(
                OdeSystem::new_with_parameters("clock", &["q"], &["q"], &bindings),
                Err(SolverError::InvalidInput(_))
            ),
            "{bindings:?}"
        );
    }
}

#[test]
fn every_constructor_rejects_missing_bindings() {
    for result in [
        OdeSystem::new_with_parameters("clock", &["q"], &["missing*q"], &[]),
        OdeSystem::autonomous_with_parameters(&["q"], &["missing*q"], &[]),
        OdeSystem::linear_with_parameters("clock", &["q"], &["missing"], &["0"], &[]),
        OdeSystem::linear_with_parameters("clock", &["q"], &["0"], &["missing"], &[]),
        OdeSystem::split_with_parameters("clock", &["q"], &["missing"], &["0"], &[]),
        OdeSystem::split_with_parameters("clock", &["q"], &["0"], &["missing*q"], &[]),
    ] {
        assert!(
            matches!(result, Err(SolverError::Evaluation { message, .. })
            if message.contains("'missing'"))
        );
    }
    // Do not hide undeclared symbols by simplifying zero products first.
    assert!(
        OdeSystem::autonomous_with_parameters(&["q"], &["zero*missing"], &[("zero", 0.0)]).is_err()
    );
}

#[test]
fn parameter_binding_preserves_structured_system_restrictions() {
    assert!(
        OdeSystem::linear_with_parameters("clock", &["q"], &["rate*q"], &["0"], &[("rate", 2.0)])
            .is_err()
    );
    assert!(
        OdeSystem::linear_with_parameters("clock", &["q"], &["0"], &["rate*q"], &[("rate", 2.0)])
            .is_err()
    );
    for matrix in [
        "rate*clock",
        "rate*q",
        "1/zero",
        "sqrt(-rate)",
        "1e308*rate",
    ] {
        assert!(
            OdeSystem::split_with_parameters(
                "clock",
                &["q"],
                &[matrix],
                &["0"],
                &[("rate", 2.0), ("zero", 0.0)]
            )
            .is_err(),
            "{matrix}"
        );
    }
    assert!(OdeSystem::split_with_parameters("clock", &["q"], &[], &["0"], &[]).is_err());
}
