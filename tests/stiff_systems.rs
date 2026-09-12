//! Multiscale regressions: accuracy AND progress beyond the fast time scale.
use cauchy_ode::{Method, OdeSystem, Solution, Solver, SolverError};
use std::collections::HashMap;

fn jacobian(system: &OdeSystem<'_>, state: &[f64]) -> Vec<Vec<f64>> {
    let args: HashMap<_, _> = system
        .variables()
        .iter()
        .copied()
        .zip(state.iter().copied())
        .collect();
    system
        .expressions()
        .iter()
        .map(|rhs| {
            system
                .variables()
                .iter()
                .map(|name| rhs.derive(name).simplify().evaluate(&args).unwrap())
                .collect()
        })
        .collect()
}

// Stable roots for a 2x2 matrix with real, negative, widely separated roots.
// det/fast avoids catastrophic cancellation in (trace + sqrt(discriminant))/2.
fn negative_eigenvalues(a: f64, b: f64, c: f64, d: f64) -> [f64; 2] {
    let trace = a + d;
    let det = a * d - b * c;
    let fast = 0.5 * (trace - (trace * trace - 4.0 * det).sqrt());
    let slow = det / fast;
    assert!(fast.is_finite() && slow.is_finite() && fast < slow && slow < 0.0);
    [fast, slow]
}

fn solver(method: Method) -> Solver {
    Solver {
        method,
        initial_step: 1e-10,
        min_step: 1e-15,
        max_step: 0.1,
        tolerance: 1e-9,
        max_steps: 20_000,
        ..Solver::default()
    }
}

fn check_mesh(solution: &Solution, end: f64, method: Method) {
    assert_eq!(solution.times[0], 0.0);
    assert_eq!(*solution.times.last().unwrap(), end, "{method:?}");
    assert_eq!(solution.times.len(), solution.states.len());
    assert_eq!(solution.times.len(), solution.recommended_steps.len());
    assert!(solution.times.iter().all(|t| t.is_finite()));
    assert!(solution.times.windows(2).all(|t| t[1] > t[0]));
    assert!(solution.states.iter().flatten().all(|v| v.is_finite()));
    eprintln!("{method:?}: {} steps to t={end}", solution.times.len() - 1);
}

#[test]
fn coupled_linear_system_with_eighteen_decades_of_eigenvalues() {
    // Constructed modal test: x=u+v, y=v, z=w, with modal rates
    // {-1e9, -1, -1e-9}. All modes are excited, including the fast layer.
    // Exact trajectory: (exp(-1e9*t)+exp(-t), exp(-t), exp(-1e-9*t)).
    for method in [
        Method::LawsonLinear,
        Method::LawsonSplit,
        Method::Lawson,
        Method::Rosenbrock,
        Method::RosenbrockAutonomous,
    ] {
        let system = match method {
            Method::LawsonLinear => OdeSystem::linear(
                "t",
                &["x", "y", "z"],
                &["-1e9", "0", "0", "999999999", "-1", "0", "0", "0", "-1e-9"],
                &["0", "0", "0"],
            ),
            Method::LawsonSplit => OdeSystem::split(
                "t",
                &["x", "y", "z"],
                &[-1e9, 0.0, 0.0, 999999999.0, -1.0, 0.0, 0.0, 0.0, -1e-9],
                &["0", "0", "0"],
            ),
            _ => OdeSystem::autonomous(&["x", "y", "z"], &["-1e9*x+999999999*y", "-y", "-1e-9*z"]),
        }
        .unwrap();
        let j = jacobian(&system, &[2.0, 1.0, 1.0]);
        // Upper-triangular Jacobian: eigenvalues are exactly its diagonal.
        assert_eq!([j[0][0], j[1][1], j[2][2]], [-1e9, -1.0, -1e-9]);
        assert_eq!([j[1][0], j[2][0], j[2][1]], [0.0; 3]);
        let solution = solver(method)
            .solve_problem(&system, 0.0, &[2.0, 1.0, 1.0], 1.0)
            .unwrap_or_else(|e| panic!("{method:?}: {e}"));
        check_mesh(&solution, 1.0, method);
        assert!(
            solution.times.iter().any(|&t| t > 0.0 && t < 1e-9),
            "{method:?}: initial fast layer was not sampled"
        );
        let mut error: f64 = 0.0;
        for (&t, y) in solution.times.iter().zip(&solution.states) {
            let exact = [(-1e9 * t).exp() + (-t).exp(), (-t).exp(), (-1e-9 * t).exp()];
            for (actual, expected) in y.iter().zip(exact) {
                error = error.max((actual - expected).abs());
            }
        }
        eprintln!("linear {method:?}: max error={error:e}");
        assert!(error < 1e-6, "{method:?}: max error {error:e}");
        assert!(solution.times.windows(2).any(|t| t[1] - t[0] > 1e-3));
    }
}

#[test]
fn semilinear_kaps_problem_with_nine_decades_of_eigenvalues() {
    // Kaps' problem, epsilon=1e-9; exact solution (exp(-2t), exp(-t)).
    // https://parallel-in-time.org/pySDC/coverage/z_91faa57f8583c837_odeSystem_py.html
    // B=[[-(1/eps+2),0],[1,-1]], u(x,y)=[y^2/eps,-y^2].
    // J=[[-(1/eps+2),2*y/eps],[1,-1-2*y]], with eigenvalues
    // approximately -1e9 and -1 along the trajectory.
    for method in [
        Method::LawsonSplit,
        Method::Lawson,
        Method::Rosenbrock,
        Method::RosenbrockAutonomous,
    ] {
        let system = if method == Method::LawsonSplit {
            OdeSystem::split(
                "t",
                &["x", "y"],
                &[-1000000002.0, 1.0, 0.0, -1.0],
                &["1e9*y^2", "-y^2"],
            )
        } else {
            OdeSystem::autonomous(&["x", "y"], &["-1000000002*x+1e9*y^2", "x-y-y^2"])
        }
        .unwrap();
        for state in [[1.0, 1.0], [(-2.0_f64).exp(), (-1.0_f64).exp()]] {
            let j = jacobian(&system, &state);
            let [fast, slow] = negative_eigenvalues(j[0][0], j[0][1], j[1][0], j[1][1]);
            assert!(fast.abs() / slow.abs() > 9e8);
        }
        if matches!(method, Method::LawsonSplit | Method::Lawson) {
            // This split leaves O(1/epsilon) derivatives in the explicit remainder.
            // A constant stiff matrix alone does not make this a suitable split;
            // general Lawson also remains fast-scale limited on this benchmark.
            let limited = Solver {
                max_steps: 2000,
                ..solver(method)
            };
            assert!(
                matches!(limited.solve_problem(&system, 0.0, &[1.0, 1.0], 1.0),
                Err(SolverError::StepLimitExceeded { time }) if time < 1e-3)
            );
            continue;
        }
        let solution = solver(method)
            .solve_problem(&system, 0.0, &[1.0, 1.0], 1.0)
            .unwrap_or_else(|e| panic!("{method:?}: {e}"));
        check_mesh(&solution, 1.0, method);
        let mut error: f64 = 0.0;
        for (&t, y) in solution.times.iter().zip(&solution.states) {
            error = error
                .max((y[0] - (-2.0 * t).exp()).abs())
                .max((y[1] - (-t).exp()).abs());
        }
        eprintln!("Kaps {method:?}: max error={error:e}");
        assert!(error < 1e-6, "{method:?}: max error {error:e}");
        assert!(solution.times.windows(2).any(|t| t[1] - t[0] > 1e-4));
    }
}

#[test]
fn nonlinear_robertson_chemical_kinetics() {
    // SUNDIALS Robertson kinetics example, including its independent t=4e10 reference:
    // https://github.com/LLNL/sundials/blob/main/examples/cvode/serial/cvRoberts_dns.c
    // A zero eigenvalue comes from mass conservation; the two nonzero
    // eigenvalues separate dramatically as the reaction evolves.
    let system = OdeSystem::autonomous(
        &["x", "y", "z"],
        &["-0.04*x+1e4*y*z", "0.04*x-1e4*y*z-3e7*y^2", "3e7*y^2"],
    )
    .unwrap();
    let reference = [
        5.208349589433733e-8,
        2.083339942979567e-13,
        9.999999479162978e-1,
    ];
    for method in [
        Method::Lawson,
        Method::Rosenbrock,
        Method::RosenbrockAutonomous,
    ] {
        let solver = Solver {
            max_step: 1e9,
            tolerance: 1e-12,
            ..solver(method)
        };
        if method == Method::Lawson {
            let limited = Solver {
                max_steps: 2000,
                ..solver
            };
            assert!(
                matches!(limited.solve_problem(&system, 0.0, &[1.0, 0.0, 0.0], 4e10),
                Err(SolverError::StepLimitExceeded { time }) if time < 1.0)
            );
            continue;
        }
        let solution = solver
            .solve_problem(&system, 0.0, &[1.0, 0.0, 0.0], 4e10)
            .unwrap_or_else(|e| panic!("{method:?}: {e}"));
        check_mesh(&solution, 4e10, method);
        for y in &solution.states {
            assert!(
                y.iter().all(|v| *v >= -1e-10),
                "{method:?}: negative concentration {y:?}"
            );
            assert!(
                (y.iter().sum::<f64>() - 1.0).abs() < 1e-7,
                "{method:?}: mass {y:?}"
            );
        }
        eprintln!(
            "Robertson {method:?}: {:?}",
            solution.states.last().unwrap()
        );
        let final_state = solution.states.last().unwrap();
        let j = jacobian(&system, final_state);
        // Restrict to x+y+z=1 (eliminate z); the third eigenvalue is zero.
        let [fast, slow] = negative_eigenvalues(
            j[0][0] - j[0][2],
            j[0][1] - j[0][2],
            j[1][0] - j[1][2],
            j[1][1] - j[1][2],
        );
        assert!(
            fast.abs() / slow.abs() > 1e12,
            "{method:?}: Robertson nonzero eigenvalues {fast:e}, {slow:e}"
        );
        for (&actual, expected) in solution.states.last().unwrap().iter().zip(reference) {
            assert!(
                (actual - expected).abs() < 1e-3 * expected.abs() + 1e-18,
                "{method:?}: {actual:e} != {expected:e}"
            );
        }
        assert!(solution.times.windows(2).any(|t| t[1] - t[0] > 1e6));
    }
}

#[test]
fn explicit_england_is_limited_by_the_fast_linear_mode() {
    let system =
        OdeSystem::autonomous(&["x", "y", "z"], &["-1e9*x+999999999*y", "-y", "-1e-9*z"]).unwrap();
    let limited = Solver {
        max_steps: 2000,
        ..solver(Method::England)
    };
    assert!(
        matches!(limited.solve_problem(&system, 0.0, &[2.0, 1.0, 1.0], 1.0),
        Err(SolverError::StepLimitExceeded { time }) if time < 1e-3)
    );
}
