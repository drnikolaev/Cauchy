//! `cargo run --release --example thomas`
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run(support::Demo {
        slug: "thomas",
        system_name: "Thomas",
        shape: "labyrinth",
        index: "03",
        description: "Three sine waves, cyclically coupled.<br>A winding path through phase space.",
        equations: ["sin(y)-0.208*x", "sin(z)-0.208*y", "sin(x)-0.208*z"],
        equation_html: "<div><span>ẋ</span> = sin y − 0.208x</div><div><span>ẏ</span> = sin z − 0.208y</div><div><span>ż</span> = sin x − 0.208z</div>",
        parameter_html: "<div><small>b · damping</small><b>0.208</b></div>",
        view_config: r#"{"slug":"thomas","shape":"labyrinth","parameters":"b = 0.208","transient":100,"camera":[-0.7,-0.55],"center":[1.35,1.35,1.35],"scale":3.5,"zMin":-1.5,"zMax":4,"gridRange":4,"gridStep":1,"gridZ":-1.5,"axisMin":-1.5,"colorExponent":1}"#,
        duration: 500.0,
        // Equal coordinates stay on the invariant diagonal x=y=z. An
        // asymmetric state excites the full three-dimensional dynamics.
        initial_state: [1.0, 0.0, 0.0],
    })
}
