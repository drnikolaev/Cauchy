use cauchy_ode::{OdeSystem, Solver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system = OdeSystem::autonomous(&["x", "y"], &["cos(y)", "sin(x)"])?;
    // Start at (x, y) = (0, 0) and integrate from t = 0 to t = 10.
    let trajectory = Solver::default().solve_problem(&system, 0.0, &[0.0, 0.0], 10.0)?;

    println!("t,x,y");
    for (t, state) in trajectory.times.iter().zip(&trajectory.states) {
        println!("{t:.6},{:.6},{:.6}", state[0], state[1]);
    }
    Ok(())
}
