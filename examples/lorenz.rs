//! `cargo run --release --example lorenz`
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run(support::Demo {
        slug: "lorenz",
        system_name: "Lorenz",
        shape: "butterfly",
        index: "01",
        description: "Three equations. Two wings.<br>A trajectory that never quite repeats.",
        equations: ["10*(y-x)", "x*(28-z)-y", "x*y-8*z/3"],
        equation_html: "<div><span>ẋ</span> = 10(y − x)</div><div><span>ẏ</span> = x(28 − z) − y</div><div><span>ż</span> = xy − 8z/3</div>",
        parameter_html: "<div><small>σ</small><b>10</b></div><div><small>ρ</small><b>28</b></div><div><small>β</small><b>8/3</b></div>",
        view_config: r#"{"slug":"lorenz","shape":"butterfly","parameters":"σ = 10   ρ = 28   β = 8/3","transient":5,"camera":[-0.3,0.16],"center":[0,0,25],"scale":27,"zMax":50,"gridRange":30,"gridStep":10,"colorExponent":1}"#,
        duration: 60.0,
        initial_state: [1.0, 1.0, 1.0],
    })
}
