//! Shared runner for the offline attractor demos.
use cauchy_ode::{Method, OdeSystem, Solver};
use std::error::Error;
use std::fmt::Write;
use std::path::PathBuf;
use std::process::Command;

pub struct Demo {
    pub slug: &'static str,
    pub system_name: &'static str,
    pub shape: &'static str,
    pub index: &'static str,
    pub description: &'static str,
    pub equations: [&'static str; 3],
    pub equation_html: &'static str,
    pub parameter_html: &'static str,
    pub view_config: &'static str,
    pub duration: f64,
    pub initial_state: [f64; 3],
}

pub fn run(demo: Demo) -> Result<(), Box<dyn Error>> {
    let mut method = Method::England;
    let mut duration = demo.duration;
    let mut output =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("target/{}.html", demo.slug));
    let mut open = true;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--method" => {
                method = match args.next().ok_or("--method requires a value")?.as_str() {
                    "england" => Method::England,
                    "lawson" => Method::Lawson,
                    "rosenbrock" => Method::Rosenbrock,
                    "rosenbrock-autonomous" => Method::RosenbrockAutonomous,
                    _ => {
                        return Err(
                            "choose england, lawson, rosenbrock, or rosenbrock-autonomous".into(),
                        );
                    }
                };
            }
            "--duration" => duration = args.next().ok_or("--duration requires a value")?.parse()?,
            "--output" => output = args.next().ok_or("--output requires a path")?.into(),
            "--no-open" => open = false,
            "--help" | "-h" => {
                println!(
                    "{} {} — computed by cauchy-ode\n\n\
                    cargo run --release --example {} -- [OPTIONS]\n\n\
                    --method england|lawson|rosenbrock|rosenbrock-autonomous\n\
                    --duration SECONDS    Integration interval (default: {}, maximum: 5000000)\n\
                    --output PATH         HTML output (default: target/{}.html)\n\
                    --no-open             Generate the viewer without launching a browser",
                    demo.system_name, demo.shape, demo.slug, demo.duration, demo.slug
                );
                return Ok(());
            }
            _ => return Err(format!("unknown option: {arg}; use --help").into()),
        }
    }
    if !duration.is_finite() || duration <= 0.0 || duration > 5000000.0 {
        return Err("duration must be finite, positive, and at most 5000000".into());
    }

    let system = OdeSystem::autonomous(&["x", "y", "z"], &demo.equations)?;
    let solver = Solver {
        method,
        initial_step: 0.005,
        max_step: 0.02,
        tolerance: 1e-9,
        max_steps: 10_000_000,
        ..Solver::default()
    };
    println!(
        "Integrating the {} system with {method:?} over t = 0..{duration} …",
        demo.system_name
    );
    let solution = solver.solve_problem(&system, 0.0, &demo.initial_state, duration)?;

    // The viewer receives only finite numeric data, so neither a JSON dependency
    // nor escaping user-provided strings into executable JavaScript is needed.
    let mut data = String::with_capacity(solution.times.len() * 90);
    data.push('[');
    for (i, (&t, state)) in solution.times.iter().zip(&solution.states).enumerate() {
        if i != 0 {
            data.push(',');
        }
        write!(data, "[{t},{},{},{}]", state[0], state[1], state[2])?;
    }
    data.push(']');
    // All presentation/configuration strings below are source-controlled demo
    // constants. CLI values never enter HTML or executable JavaScript.
    let equation_label = format!(
        "dx/dt = {}, dy/dt = {}, dz/dt = {}",
        demo.equations[0], demo.equations[1], demo.equations[2]
    );
    let html = include_str!("viewer.html")
        .replace("__TRAJECTORY_DATA__", &data)
        .replace("__SYSTEM__", demo.system_name)
        .replace("__SHAPE__", demo.shape)
        .replace("__INDEX__", demo.index)
        .replace("__DESCRIPTION__", demo.description)
        .replace("__EQUATION_LABEL__", &equation_label)
        .replace("__EQUATIONS_HTML__", demo.equation_html)
        .replace("__PARAMETERS_HTML__", demo.parameter_html)
        .replace("__VIEW_CONFIG__", demo.view_config)
        .replace(
            "__INITIAL__",
            &format!(
                "({}, {}, {})",
                demo.initial_state[0], demo.initial_state[1], demo.initial_state[2]
            ),
        )
        .replace("__METHOD__", &format!("{method:?}"));
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, html)?;
    let output = output.canonicalize()?;
    println!(
        "Saved {} trajectory points to {}",
        solution.times.len(),
        output.display()
    );
    println!("Drag to rotate · Scroll to zoom · Replay to follow the trajectory");
    if open {
        let result = if cfg!(target_os = "macos") {
            Command::new("open").arg(&output).status()
        } else if cfg!(target_os = "windows") {
            Command::new("explorer").arg(&output).status()
        } else {
            Command::new("xdg-open").arg(&output).status()
        };
        if !matches!(result, Ok(status) if status.success()) {
            eprintln!(
                "Could not launch a browser. Open {} manually.",
                output.display()
            );
        }
    }
    Ok(())
}
