use cauchy_ode::{Method, OdeSystem, Solver, SolverError};

const METHODS: [Method; 6] = [
    Method::England,
    Method::Lawson,
    Method::LawsonLinear,
    Method::LawsonSplit,
    Method::Rosenbrock,
    Method::RosenbrockAutonomous,
];

fn solver(method: Method) -> Solver {
    Solver {
        method,
        ..Solver::default()
    }
}
fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{actual} != {expected} (tol={tolerance})"
    );
}

fn oscillator(method: Method) -> OdeSystem<'static> {
    match method {
        Method::LawsonLinear => {
            OdeSystem::linear("t", &["q", "v"], &["0", "-1", "1", "0"], &["0", "0"])
        }
        Method::LawsonSplit => {
            OdeSystem::split("t", &["q", "v"], &[0.0, -1.0, 1.0, 0.0], &["0", "0"])
        }
        Method::RosenbrockAutonomous => OdeSystem::autonomous(&["q", "v"], &["v", "-q"]),
        _ => OdeSystem::new("t", &["q", "v"], &["v", "-q"]),
    }
    .unwrap()
}

#[test]
fn all_six_methods_solve_coupled_systems_forward_and_backward() {
    for method in METHODS {
        let system = oscillator(method);
        for end in [2.0, -2.0] {
            let solution = solver(method)
                .solve_problem(&system, 0.0, &[1.0, 0.0], end)
                .unwrap();
            assert_eq!(*solution.times.last().unwrap(), end, "{method:?}");
            assert_eq!(solution.times.len(), solution.recommended_steps.len());
            for (&time, state) in solution.times.iter().zip(&solution.states) {
                close(state[0], time.cos(), 1e-6);
                close(state[1], -time.sin(), 1e-6);
            }
            assert!(
                solution
                    .recommended_steps
                    .iter()
                    .all(|h| h.is_finite() && h.signum() == end.signum())
            );
        }
    }
}

#[test]
fn general_methods_evaluate_time_and_state_derivatives() {
    for method in [Method::England, Method::Lawson, Method::Rosenbrock] {
        // Exact solution x=t^2; Rosenbrock needs both df/dt and df/dx.
        let solution = solver(method).solve("t+sqrt(x)", 1.0, 1.0, 2.0).unwrap();
        close(solution.states.last().unwrap()[0], 4.0, 1e-6);
    }
}

#[test]
fn lawson_and_rosenbrock_solve_stiff_non_autonomous_problem() {
    for method in [Method::Lawson, Method::Rosenbrock] {
        let solution = solver(method)
            .solve("-1000*(x-cos(t))-sin(t)", 0.0, 1.0, 1.0)
            .unwrap();
        for (&time, state) in solution.times.iter().zip(&solution.states) {
            close(state[0], time.cos(), 1e-6);
        }
    }
}

#[test]
fn autonomous_rosenbrock_handles_nonlinear_jacobian_and_scalar_aliases() {
    let solution = solver(Method::RosenbrockAutonomous)
        .solve("-x*x0", 0.0, 1.0, 1.0)
        .unwrap();
    for (&time, state) in solution.times.iter().zip(&solution.states) {
        close(state[0], 1.0 / (1.0 + time), 1e-6);
    }
}

#[test]
fn time_linear_lawson_uses_varying_matrix_and_forcing() {
    // Exact solution (t^2+1, exp(-t)), with a nonsymmetric, varying A(t).
    let system = OdeSystem::linear(
        "s",
        &["position", "velocity"],
        &["-2", "0", "s", "-3"],
        &["2*s+2*(s^2+1)-s*exp(-s)", "2*exp(-s)"],
    )
    .unwrap();
    for method in [
        Method::LawsonLinear,
        Method::Lawson,
        Method::Rosenbrock,
        Method::England,
    ] {
        let solution = solver(method)
            .solve_problem(&system, 0.0, &[1.0, 1.0], 1.0)
            .unwrap();
        for (&time, state) in solution.times.iter().zip(&solution.states) {
            close(state[0], time * time + 1.0, 1e-6);
            close(state[1], (-time).exp(), 1e-6);
        }
    }
}

#[test]
fn split_lawson_uses_constant_matrix_and_nonlinear_remainder() {
    let system = OdeSystem::split(
        "s",
        &["a", "b"],
        &[-20.0, 0.0, 7.0, -3.0],
        &["19*exp(-s)-7*b+(a-exp(-s))^2", "exp(-2*s)"],
    )
    .unwrap();
    for method in [
        Method::LawsonSplit,
        Method::Lawson,
        Method::Rosenbrock,
        Method::England,
    ] {
        let solution = solver(method)
            .solve_problem(&system, 0.0, &[1.0, 1.0], 1.0)
            .unwrap();
        for (&time, state) in solution.times.iter().zip(&solution.states) {
            close(state[0], (-time).exp(), 1e-6);
            close(state[1], (-2.0 * time).exp(), 1e-6);
        }
    }
}

#[test]
fn all_methods_handle_a_short_final_step() {
    for method in METHODS {
        let solver = Solver {
            initial_step: 0.1,
            min_step: 0.01,
            ..solver(method)
        };
        let solution = solver
            .solve_problem(&oscillator(method), 0.0, &[1.0, 0.0], 0.001)
            .unwrap();
        assert_eq!(*solution.times.last().unwrap(), 0.001);
        close(solution.states.last().unwrap()[0], 0.001_f64.cos(), 1e-8);
    }
}

#[test]
fn all_methods_report_minimum_step_failure() {
    for method in METHODS {
        let solver = Solver {
            initial_step: 1.0,
            min_step: 1.0,
            max_step: 1.0,
            tolerance: 1e-15,
            ..solver(method)
        };
        let system = match method {
            Method::LawsonLinear => OdeSystem::linear("t", &["x"], &["t^2"], &["1"]),
            Method::LawsonSplit => OdeSystem::split("t", &["x"], &[0.0], &["-x^2"]),
            Method::RosenbrockAutonomous => OdeSystem::autonomous(&["x"], &["-x^2"]),
            _ => OdeSystem::new("t", &["x"], &["-x^2"]),
        }
        .unwrap();
        assert!(
            matches!(
                solver.solve_problem(&system, 0.0, &[1.0], 1.0),
                Err(SolverError::StepSizeTooSmall { .. })
            ),
            "{method:?}"
        );
    }
}

#[test]
fn matrix_failures_are_not_misreported_as_tolerance_failures() {
    for method in [Method::Rosenbrock, Method::RosenbrockAutonomous] {
        // I-h*J is singular at h=1, J=1.
        let solver = Solver {
            initial_step: 1.0,
            max_step: 1.0,
            ..solver(method)
        };
        assert_eq!(
            solver.solve("x", 0.0, 1.0, 1.0),
            Err(SolverError::FortranFailure { info: 2001 })
        );
    }
    let system = OdeSystem::split("t", &["x"], &[1e308], &["0"]).unwrap();
    assert!(matches!(
        solver(Method::LawsonSplit).solve_problem(&system, 0.0, &[1.0], 1.0),
        Err(SolverError::FortranFailure { .. })
    ));
}

#[test]
fn callback_errors_propagate_from_each_solver_family() {
    for method in METHODS {
        let system = match method {
            Method::LawsonLinear => OdeSystem::linear("t", &["x"], &["0"], &["sqrt(-1)"]),
            Method::LawsonSplit => OdeSystem::split("t", &["x"], &[0.0], &["sqrt(-1)"]),
            Method::RosenbrockAutonomous => OdeSystem::autonomous(&["x"], &["sqrt(-1)"]),
            _ => OdeSystem::new("t", &["x"], &["sqrt(-1)"]),
        }
        .unwrap();
        assert!(
            matches!(
                solver(method).solve_problem(&system, 0.0, &[1.0], 1.0),
                Err(SolverError::NonFiniteDerivative { .. })
            ),
            "{method:?}"
        );
    }
}

#[test]
fn rejects_invalid_system_forms_and_incompatible_methods() {
    assert!(OdeSystem::new("t", &["NaN"], &["NaN"]).is_err());
    assert!(OdeSystem::autonomous(&["x"], &["t+x"]).is_err());
    assert!(OdeSystem::linear("t", &["x"], &["x"], &["0"]).is_err());
    assert!(OdeSystem::linear("t", &["x"], &["1"], &["x"]).is_err());
    assert!(OdeSystem::split("t", &["x"], &[f64::NAN], &["0"]).is_err());
    assert!(OdeSystem::linear("t", &["x", "y"], &["1"], &["0", "0"]).is_err());
    assert!(OdeSystem::new("t", &["x", "x"], &["0", "0"]).is_err());
    assert!(OdeSystem::new("x", &["x"], &["0"]).is_err());
    let general = OdeSystem::new("t", &["x"], &["x"]).unwrap();
    for method in [
        Method::LawsonLinear,
        Method::LawsonSplit,
        Method::RosenbrockAutonomous,
    ] {
        assert!(matches!(
            solver(method).solve_problem(&general, 0.0, &[1.0], 1.0),
            Err(SolverError::IncompatibleMethod { .. })
        ));
    }
}

#[test]
fn rejects_non_finite_jacobian_time_derivative_and_matrix_callbacks() {
    assert!(matches!(
        solver(Method::Lawson).solve("sqrt(x)", 0.0, 0.0, 1.0),
        Err(SolverError::NonFiniteDerivative { .. })
    ));
    assert!(matches!(
        solver(Method::Rosenbrock).solve("sqrt(t)+x", 0.0, 1.0, 1.0),
        Err(SolverError::NonFiniteDerivative { .. })
    ));
    let system = OdeSystem::linear("t", &["x"], &["sqrt(-1)"], &["0"]).unwrap();
    assert!(matches!(
        solver(Method::LawsonLinear).solve_problem(&system, 0.0, &[1.0], 1.0),
        Err(SolverError::NonFiniteDerivative { .. })
    ));
}

#[test]
fn all_methods_are_independent_across_threads() {
    let handles: Vec<_> = METHODS
        .into_iter()
        .map(|method| {
            std::thread::spawn(move || {
                for _ in 0..4 {
                    let solution = solver(method)
                        .solve_problem(&oscillator(method), 0.0, &[1.0, 0.0], 0.5)
                        .unwrap();
                    close(solution.states.last().unwrap()[0], 0.5_f64.cos(), 1e-6);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn larger_systems_exercise_quadratic_workspaces() {
    let names: Vec<_> = (0..12).map(|i| format!("v{i}")).collect();
    let rhs: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(i, name)| format!("-{}*{name}", i + 1))
        .collect();
    let names: Vec<_> = names.iter().map(String::as_str).collect();
    let rhs: Vec<_> = rhs.iter().map(String::as_str).collect();
    let general = OdeSystem::new("t", &names, &rhs).unwrap();
    let autonomous = OdeSystem::autonomous(&names, &rhs).unwrap();
    let mut matrix = vec![0.0; 144];
    for i in 0..12 {
        matrix[i * 12 + i] = -(i as f64 + 1.0);
    }
    let split = OdeSystem::split("t", &names, &matrix, &["0"; 12]).unwrap();
    let matrix: Vec<_> = matrix.iter().map(ToString::to_string).collect();
    let matrix: Vec<_> = matrix.iter().map(String::as_str).collect();
    let linear = OdeSystem::linear("t", &names, &matrix, &["0"; 12]).unwrap();
    for method in METHODS {
        let system = match method {
            Method::LawsonLinear => &linear,
            Method::LawsonSplit => &split,
            Method::RosenbrockAutonomous => &autonomous,
            _ => &general,
        };
        let solution = solver(method)
            .solve_problem(system, 0.0, &[1.0; 12], 0.5)
            .unwrap();
        for (i, &value) in solution.states.last().unwrap().iter().enumerate() {
            close(value, (-(i as f64 + 1.0) * 0.5).exp(), 1e-6);
        }
    }
}
