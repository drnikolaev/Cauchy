// SPDX-License-Identifier: BSL-1.0
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE or https://www.boost.org/LICENSE_1_0.txt.

//! Compare all supported system/method combinations against exact solutions.
//! `cargo run --release --example precision -- --no-open`
use cauchy_ode::{Method, OdeSystem, Solution, Solver};
use std::{error::Error, fmt::Write, path::PathBuf, process::Command};

const N: usize = 10;
const END: f64 = 10.0;
const INITIAL_STEP: f64 = 0.01;
const MAX_STEP: f64 = 1.0;
const TOLERANCE: f64 = 1e-12;
const METHODS: [Method; 6] = [
    Method::England,
    Method::Lawson,
    Method::LawsonLinear,
    Method::LawsonSplit,
    Method::Rosenbrock,
    Method::RosenbrockAutonomous,
];
const COLORS: [&str; 6] = [
    "#2563eb", "#d97706", "#059669", "#9333ea", "#dc2626", "#0891b2",
];

fn exact(kind: usize, i: usize, t: f64) -> f64 {
    if kind == 2 {
        let angle = (0.2 + 0.04 * (i / 2) as f64) * t;
        if i % 2 == 0 { angle.cos() } else { angle.sin() }
    } else {
        2.0 + ((0.2 + 0.02 * i as f64) * t).sin()
    }
}

struct Run {
    method: Method,
    color: &'static str,
    solution: Solution,
    errors: Vec<[f64; N]>,
}

// Render actual accepted times without interpolation. Zero errors are clipped
// only for the logarithmic plot; the table and CSV retain the original values.
fn plot(runs: &[Run], component: Option<usize>) -> String {
    let title = component.map_or_else(
        || "Maximum over all 10 components".into(),
        |i| format!("Component x{i}"),
    );
    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 760 560' role='img' aria-label='{title}: absolute error and accepted step size versus time'><rect width='760' height='560' fill='white'/><text x='80' y='23' font-size='17'>{title}</text>"
    );
    for exponent in -16..=0 {
        let y = 250.0 - (exponent + 16) as f64 / 16.0 * 210.0;
        write!(svg, "<path d='M80 {y}H730' stroke='#e2e8f0'/>").unwrap();
        if exponent % 2 == 0 {
            write!(
                svg,
                "<text x='70' y='{}' text-anchor='end' font-size='12'>1e{exponent}</text>",
                y + 4.0
            )
            .unwrap();
        }
    }
    for t in 0..=10 {
        let x = 80 + t * 65;
        write!(
            svg,
            "<text x='{x}' y='270' text-anchor='middle' font-size='12'>{t}</text>"
        )
        .unwrap();
    }
    svg.push_str("<text x='400' y='292' font-size='13'>Time t</text><text transform='translate(18 200) rotate(-90)' font-size='13'>Absolute error (log scale)</text>");
    svg.push_str(
        "<text x='80' y='325' font-size='15'>Accepted step size · initial = 0.01 · cap = 1</text>",
    );
    // A linear axis keeps ordinary adaptations visible even when a final
    // endpoint step is close to roundoff. Each h spans its own interval.
    let maximum_step = runs
        .iter()
        .flat_map(|run| run.solution.times.windows(2))
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(INITIAL_STEP, f64::max);
    let step_y = |h: f64| 470.0 - h / (maximum_step * 1.1) * 120.0;
    for tick in 0..=4 {
        let h = maximum_step * tick as f64 / 4.0;
        let y = step_y(h);
        write!(svg, "<path d='M80 {y}H730' stroke='#e2e8f0'/>").unwrap();
        write!(
            svg,
            "<text x='70' y='{}' text-anchor='end' font-size='12'>{h:.3}</text>",
            y + 4.0
        )
        .unwrap();
    }
    for t in 0..=10 {
        let x = 80 + t * 65;
        write!(
            svg,
            "<text x='{x}' y='490' text-anchor='middle' font-size='12'>{t}</text>"
        )
        .unwrap();
    }
    svg.push_str("<text x='400' y='512' font-size='13'>Time t</text><text transform='translate(18 455) rotate(-90)' font-size='13'>Step h (linear scale)</text>");
    for (index, run) in runs.iter().enumerate() {
        let mut path = String::new();
        for (k, (&t, errors)) in run.solution.times.iter().zip(&run.errors).enumerate() {
            let error =
                component.map_or_else(|| errors.iter().copied().fold(0.0, f64::max), |i| errors[i]);
            let x = 80.0 + 650.0 * t / END;
            let y = 250.0 - (error.clamp(1e-16, 1.0).log10() + 16.0) / 16.0 * 210.0;
            write!(path, "{}{x:.3},{y:.3}", if k == 0 { 'M' } else { 'L' }).unwrap();
        }
        write!(svg, "<path d='{path}' fill='none' stroke='{}' stroke-width='1.4'><title>{:?}</title></path>", run.color, run.method).unwrap();
        let mut step_path = String::new();
        for (k, pair) in run.solution.times.windows(2).enumerate() {
            let x0 = 80.0 + 650.0 * pair[0] / END;
            let x1 = 80.0 + 650.0 * pair[1] / END;
            let y = step_y((pair[1] - pair[0]).abs());
            if k == 0 {
                write!(step_path, "M{x0:.3},{y:.3}").unwrap();
            } else {
                write!(step_path, "V{y:.3}").unwrap();
            }
            write!(step_path, "H{x1:.3}").unwrap();
        }
        write!(svg, "<path d='{step_path}' fill='none' stroke='{}' stroke-width='1.4'><title>{:?}: actual accepted step sizes</title></path>", run.color, run.method).unwrap();
        let x = 80 + index * 165;
        write!(svg, "<path d='M{x} 536h15' stroke='{}' stroke-width='2'/><text x='{}' y='540' font-size='11'>{:?}</text>", run.color, x + 20, run.method).unwrap();
    }
    svg.push_str("</svg>");
    svg
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/precision");
    let mut open = true;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-open" => open = false,
            "--output" => output = args.next().ok_or("--output requires a directory")?.into(),
            "--help" | "-h" => {
                println!(
                    "cargo run --release --example precision -- [--no-open] [--output DIRECTORY]\nWrites index.html, 44 SVG error plots, and errors.csv (default: target/precision)."
                );
                return Ok(());
            }
            _ => return Err(format!("unknown option: {arg}").into()),
        }
    }
    std::fs::create_dir_all(&output)?;
    let names: Vec<String> = (0..N).map(|i| format!("x{i}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut html = String::from(
        "<!doctype html><html lang='en'><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>ODE precision comparison</title><style>body{font:16px system-ui;color:#172033;background:#f1f5f9;max-width:1100px;margin:40px auto;padding:0 20px}section{background:white;padding:24px;margin:24px 0;border-radius:12px}img{width:100%;height:auto}table{border-collapse:collapse;width:100%;font-variant-numeric:tabular-nums}td,th{text-align:left;padding:8px;border-bottom:1px solid #ddd}code{font-size:14px}summary{cursor:pointer;padding:15px}a{color:#2563eb}</style><h1>Ten equations. Four system types.</h1><p>Absolute error against an analytic solution at every accepted time. All 15 compatible method/type combinations use t = 0…10, initial step 0.01, maximum step 1, local tolerance 1e-12, and the default mixed relative/absolute error control. The adaptive controller chooses step sizes within these bounds; there is no target step count. This compares accuracy under common controls, not computational cost or general method rankings.</p><p>Logarithmic error plots share a 1e-16…1 scale. Errors below 1e-16 (including zero at the initial point) are displayed at the floor. Near that floor, floating-point evaluation of the analytic solution also affects the comparison. Tables and <a href='errors.csv'>CSV data</a> retain unclipped errors.</p>",
    );
    let mut csv = String::from("type,method,step,time,component,numerical,exact,absolute_error\n");
    for kind in 1..=4 {
        let mut rhs = vec![String::new(); N];
        let mut matrix = vec!["0".to_string(); N * N];
        let mut constant = vec![0.0; N * N];
        let q: Vec<String> = (0..N)
            .map(|i| format!("(2+sin({}*t))", 0.2 + 0.02 * i as f64))
            .collect();
        for i in 0..N {
            let next = (i + 1) % N;
            let w = 0.2 + 0.02 * i as f64;
            let derivative = format!("{w}*cos({w}*t)");
            rhs[i] = match kind {
                1 => format!(
                    "{derivative}-(1+0.1*sin(t))*(x{i}-{})+0.05*(x{next}-{})^2",
                    q[i], q[next]
                ),
                2 => {
                    let a = i / 2 * 2;
                    let b = a + 1;
                    let omega = 0.2 + 0.04 * (i / 2) as f64;
                    let rotation = if i % 2 == 0 {
                        format!("-{omega}*x{b}")
                    } else {
                        format!("{omega}*x{a}")
                    };
                    format!("{rotation}+0.1*(1-x{a}^2-x{b}^2)*x{i}")
                }
                3 => {
                    matrix[i * N + i] = "-(1+0.1*sin(t))".into();
                    matrix[next * N + i] = "0.1".into();
                    format!("{derivative}+(1+0.1*sin(t))*{}-0.1*{}", q[i], q[next])
                }
                4 => {
                    constant[i * N + i] = -1.0;
                    constant[next * N + i] = 0.1;
                    format!(
                        "{derivative}+{}-0.1*{}+0.05*(x{next}-{})^2",
                        q[i], q[next], q[next]
                    )
                }
                _ => unreachable!(),
            };
        }
        let expressions: Vec<&str> = rhs.iter().map(String::as_str).collect();
        let entries: Vec<&str> = matrix.iter().map(String::as_str).collect();
        let system = match kind {
            1 => OdeSystem::general("t", &names, &expressions)?,
            2 => OdeSystem::autonomous(&names, &expressions)?,
            3 => OdeSystem::linear("t", &names, &entries, &expressions)?,
            4 => OdeSystem::split("t", &names, &constant, &expressions)?,
            _ => unreachable!(),
        };
        let description = match kind {
            1 => "General: xᵢ′ = qᵢ′ − (1 + 0.1 sin t)(xᵢ − qᵢ) + 0.05(xⱼ − qⱼ)².",
            2 => {
                "Autonomous: for k = 0…4, a = x₂ₖ, b = x₂ₖ₊₁, ω = 0.2 + 0.04k: a′ = −ωb + 0.1(1 − a² − b²)a; b′ = ωa + 0.1(1 − a² − b²)b. Exact: a = cos(ωt), b = sin(ωt); initially (a,b) = (1,0). The radial nonlinear terms vanish on these unit circles."
            }
            3 => {
                "Linear: xᵢ′ = −(1 + 0.1 sin t)xᵢ + 0.1xⱼ + φᵢ(t), where φᵢ = qᵢ′ + (1 + 0.1 sin t)qᵢ − 0.1qⱼ."
            }
            4 => {
                "Constant linear part: (Bx)ᵢ = −xᵢ + 0.1xⱼ; uᵢ = qᵢ′ + qᵢ − 0.1qⱼ + 0.05(xⱼ − qⱼ)²; x′ = Bx + u."
            }
            _ => unreachable!(),
        };
        write!(html, "<section><h2>Type {kind}</h2><p>{description}</p>")?;
        if kind != 2 {
            html.push_str("<p>For i = 0…9, j = (i + 1) mod 10, ωᵢ = 0.2 + 0.02i, qᵢ(t) = 2 + sin(ωᵢt), qᵢ′ = ωᵢ cos(ωᵢt). Exact: xᵢ = qᵢ, initially xᵢ = 2. Substitution cancels the correction terms and gives xᵢ′ = qᵢ′.</p>");
        }
        html.push_str("<table><tr><th>Method / plot color</th><th>Accepted steps</th><th>Peak error</th><th>Final max error</th></tr>");
        let initial: Vec<f64> = (0..N).map(|i| exact(kind, i, 0.0)).collect();
        let mut runs = Vec::new();
        for (method, color) in METHODS.into_iter().zip(COLORS) {
            if !method.supports(system.kind()) {
                continue;
            }
            let solver = Solver {
                method,
                initial_step: INITIAL_STEP,
                max_step: MAX_STEP,
                tolerance: TOLERANCE,
                ..Solver::default()
            };
            let solution = solver.solve_problem(&system, 0.0, &initial, END)?;
            let mut errors = Vec::new();
            for (step, (&t, state)) in solution.times.iter().zip(&solution.states).enumerate() {
                let mut error = [0.0; N];
                for i in 0..N {
                    let expected = exact(kind, i, t);
                    error[i] = (state[i] - expected).abs();
                    writeln!(
                        csv,
                        "{kind},{method:?},{step},{t:.17e},{i},{:.17e},{expected:.17e},{:.17e}",
                        state[i], error[i]
                    )?;
                }
                errors.push(error);
            }
            let peak = errors.iter().flatten().copied().fold(0.0, f64::max);
            let final_error = errors.last().unwrap().iter().copied().fold(0.0, f64::max);
            let steps = solution.times.len() - 1;
            // Catch changes that defeat the purpose of this reproducible demo.
            if !peak.is_finite() || peak >= 1e-3 || solution.times.last() != Some(&END) {
                return Err(format!(
                    "Type {kind} / {method:?}: endpoint not reached or excessive error ({peak:e})"
                )
                .into());
            }
            println!(
                "Type {kind} {method:?}: {steps} steps, peak {peak:.3e}, final {final_error:.3e}"
            );
            write!(
                html,
                "<tr><td style='color:{color}'>{method:?}</td><td>{steps}</td><td>{peak:.3e}</td><td>{final_error:.3e}</td></tr>"
            )?;
            runs.push(Run {
                method,
                color,
                solution,
                errors,
            });
        }
        html.push_str("</table><p>Every figure includes an accepted-step panel: h = |tₙ − tₙ₋₁| over each integration interval, with a separate linear axis. All components share the same steps. Method curves overlap when their steps agree; the final step may be shortened to reach the endpoint. These are actual accepted steps, not recommended next steps.</p>");
        for component in std::iter::once(None).chain((0..N).map(Some)) {
            if component == Some(0) {
                html.push_str("<details><summary>Errors for each of the 10 components</summary>");
            }
            let suffix = component.map_or_else(|| "max".into(), |i| format!("x{i}"));
            let filename = format!("type{kind}-{suffix}.svg");
            std::fs::write(output.join(&filename), plot(&runs, component))?;
            write!(
                html,
                "<a href='{filename}'><img src='{filename}' alt='Type {kind}, {suffix}: absolute error and accepted step size versus time' loading='lazy'></a>"
            )?;
        }
        html.push_str("</details></section>");
    }
    html.push_str("<footer>Distributed under the <a href='https://www.boost.org/LICENSE_1_0.txt'>Boost Software License, Version 1.0</a>.</footer></html>");
    std::fs::write(output.join("errors.csv"), csv)?;
    std::fs::write(output.join("index.html"), html)?;
    let index = output.join("index.html").canonicalize()?;
    println!("Report: {}", index.display());
    if open {
        let browser = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(target_os = "windows") {
            "explorer"
        } else {
            "xdg-open"
        };
        if !matches!(Command::new(browser).arg(&index).status(), Ok(s) if s.success()) {
            eprintln!("Open {} in a browser to view the plots.", index.display());
        }
    }
    Ok(())
}
