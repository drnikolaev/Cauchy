//! `cargo run --release --example rossler`
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run(support::Demo {
        slug: "rossler",
        system_name: "Rössler",
        shape: "ribbon",
        index: "02",
        description: "A spiral stretches, rises, and folds.<br>Order gives way to a ribbon of chaos.",
        equations: ["-y-z", "x+0.2*y", "0.2+z*(x-5.7)"],
        equation_html: "<div><span>ẋ</span> = −y − z</div><div><span>ẏ</span> = x + 0.2y</div><div><span>ż</span> = 0.2 + z(x − 5.7)</div>",
        parameter_html: "<div><small>a</small><b>0.2</b></div><div><small>b</small><b>0.2</b></div><div><small>c</small><b>5.7</b></div>",
        view_config: r#"{"slug":"rossler","shape":"ribbon","parameters":"a = 0.2   b = 0.2   c = 5.7","transient":50,"camera":[-0.35,-0.95],"center":[0,-1,6],"scale":14,"zMax":25,"gridRange":15,"gridStep":5,"colorExponent":0.35}"#,
        duration: 300.0,
        initial_state: [1.0, 1.0, 1.0],
    })
}
