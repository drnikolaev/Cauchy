use cauchy_ode::{Solver, SolverError};

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn solves_natural_expression_using_stage_time_and_state() {
    // x(t) = t^2 solves x' = t + sqrt(x) for t >= 0.
    let solution = Solver::default().solve("t+sqrt(x)", 1.0, 1.0, 2.0).unwrap();
    assert_eq!(solution.times[0], 1.0);
    assert_eq!(solution.states[0], [1.0]);
    assert_eq!(*solution.times.last().unwrap(), 2.0);
    for (&time, state) in solution.times.iter().zip(&solution.states) {
        assert_close(state[0], time * time, 1e-7);
    }
}

#[test]
fn adapts_large_initial_step_for_exponential_growth() {
    let solver = Solver {
        initial_step: 1.0,
        max_step: 1.0,
        ..Solver::default()
    };
    let solution = solver.solve("x", 0.0, 1.0, 1.0).unwrap();
    assert!(solution.times[1] < 1.0);
    assert_close(
        solution.states.last().unwrap()[0],
        std::f64::consts::E,
        1e-7,
    );
}

#[test]
fn solves_coupled_harmonic_oscillator() {
    let solution = Solver::default()
        .solve_system(&["x1", "-x0"], 0.0, &[1.0, 0.0], std::f64::consts::TAU)
        .unwrap();
    for (&time, state) in solution.times.iter().zip(&solution.states) {
        assert_close(state[0], time.cos(), 1e-7);
        assert_close(state[1], -time.sin(), 1e-7);
    }
}

#[test]
fn integrates_backwards() {
    let solution = Solver::default()
        .solve("x0", 1.0, std::f64::consts::E, 0.0)
        .unwrap();
    assert_eq!(*solution.times.last().unwrap(), 0.0);
    assert!(solution.times.windows(2).all(|times| times[1] < times[0]));
    assert_close(solution.states.last().unwrap()[0], 1.0, 1e-7);
}

#[test]
fn final_step_can_be_shorter_than_minimum() {
    let solver = Solver {
        initial_step: 0.1,
        min_step: 0.1,
        max_step: 0.1,
        ..Solver::default()
    };
    for end_time in [0.25, -0.25, 0.025, -0.025] {
        let solution = solver.solve("1", 0.0, 0.0, end_time).unwrap();
        assert_eq!(*solution.times.last().unwrap(), end_time);
        assert_close(solution.states.last().unwrap()[0], end_time, 1e-14);
    }
}

#[test]
fn zero_length_interval_returns_initial_point() {
    let solution = Solver::default().solve("x", 2.0, 3.0, 2.0).unwrap();
    assert_eq!(solution.times, [2.0]);
    assert_eq!(solution.states, [vec![3.0]]);
}

#[test]
fn rejects_invalid_expression_and_unknown_variable() {
    let solver = Solver::default();
    assert!(matches!(
        solver.solve("sqrt(", 0.0, 1.0, 1.0),
        Err(SolverError::Parse { equation: 0, .. })
    ));
    let error = solver.solve("t+y", 0.0, 1.0, 1.0).unwrap_err();
    assert!(
        matches!(&error, SolverError::Evaluation { equation: 0, message } if message.contains("y"))
    );
    assert!(matches!(
        solver.solve_system(&["1", "unknown"], 0.0, &[0.0, 0.0], 1.0),
        Err(SolverError::Evaluation { equation: 1, .. })
    ));
}

#[test]
fn propagates_callback_failure_at_later_stage_and_recovers() {
    let solver = Solver::default();
    // Parsing succeeds; evaluation fails only once a stage's t exceeds 0.025.
    assert!(matches!(
        solver.solve("sqrt(0.025-t)", 0.0, 0.0, 1.0),
        Err(SolverError::NonFiniteDerivative { time, .. }) if time > 0.025
    ));
    let solution = solver.solve("1", 0.0, 0.0, 0.1).unwrap();
    assert_close(solution.states.last().unwrap()[0], 0.1, 1e-14);
}

#[test]
fn rejects_non_finite_derivatives() {
    for expression in ["sqrt(-1)", "1/(t-t)", "exp(1000)"] {
        assert!(matches!(
            Solver::default().solve(expression, 0.0, 1.0, 1.0),
            Err(SolverError::NonFiniteDerivative { equation: 0, .. })
        ));
    }
}

#[test]
fn reports_fortran_minimum_step_failure() {
    let solver = Solver {
        initial_step: 1.0,
        min_step: 1.0,
        max_step: 1.0,
        tolerance: 1e-15,
        ..Solver::default()
    };
    assert_eq!(
        solver.solve("x", 0.0, 1.0, 1.0),
        Err(SolverError::StepSizeTooSmall { time: 0.0 })
    );
}

#[test]
fn bounds_work_and_detects_time_stagnation() {
    let solver = Solver {
        max_steps: 1,
        ..Solver::default()
    };
    assert!(matches!(
        solver.solve("1", 0.0, 0.0, 1.0),
        Err(SolverError::StepLimitExceeded { .. })
    ));
    assert!(matches!(
        Solver::default().solve("1", 1e16, 0.0, 1e16 + 4.0),
        Err(SolverError::NoProgress { .. })
    ));
}

#[test]
fn validates_dimensions_times_states_and_controls() {
    let solver = Solver::default();
    for (rhs, state) in [(&[][..], &[][..]), (&["x"][..], &[1.0, 2.0][..])] {
        assert!(matches!(
            solver.solve_system(rhs, 0.0, state, 1.0),
            Err(SolverError::InvalidInput(_))
        ));
    }
    for (start, state, end) in [
        (f64::NAN, 1.0, 1.0),
        (0.0, f64::INFINITY, 1.0),
        (0.0, 1.0, f64::INFINITY),
    ] {
        assert!(matches!(
            solver.solve("x", start, state, end),
            Err(SolverError::InvalidInput(_))
        ));
    }
    for solver in [
        Solver {
            tolerance: 0.0,
            ..Solver::default()
        },
        Solver {
            min_step: -1.0,
            ..Solver::default()
        },
        Solver {
            relative_threshold: 0.0,
            ..Solver::default()
        },
        Solver {
            initial_step: 2.0,
            ..Solver::default()
        },
        Solver {
            max_step: f64::NAN,
            ..Solver::default()
        },
        Solver {
            max_steps: 0,
            ..Solver::default()
        },
    ] {
        assert!(matches!(
            solver.solve("x", 0.0, 1.0, 1.0),
            Err(SolverError::InvalidInput(_))
        ));
    }
}

#[test]
fn concurrent_solvers_have_independent_callback_contexts() {
    let threads: Vec<_> = (1..=8)
        .map(|rate| {
            std::thread::spawn(move || {
                let rhs = format!("{rate}*x");
                let solution = Solver::default().solve(&rhs, 0.0, 1.0, 0.5).unwrap();
                assert_close(
                    solution.states.last().unwrap()[0],
                    (rate as f64 * 0.5).exp(),
                    1e-5,
                );
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}
