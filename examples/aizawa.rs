//! `cargo run --release --example aizawa`
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run(support::Demo {
        slug: "aizawa",
        system_name: "Aizawa",
        shape: "vortex",
        index: "04",
        description: "A winding shell around a rising spindle.<br>Chaos traced in three dimensions.",
        // Parameters and equations:
        // https://nbodyphysics.com/chaoticmotion/html/_aizawa_8cs_source.html
        equations: [
            "(z-0.7)*x-3.5*y",
            "3.5*x+(z-0.7)*y",
            "0.6+0.95*z-z^3/3-(x^2+y^2)*(1+0.25*z)+0.1*z*x^3",
        ],
        equation_html: "<div><span>ẋ</span> = (z − b)x − dy</div><div><span>ẏ</span> = dx + (z − b)y</div><div><span>ż</span> = c + az − z³/3</div><div class=continuation>− (x² + y²)(1 + ez)</div><div class=continuation>+ fzx³</div>",
        parameter_html: "<div><small>a</small><b>0.95</b></div><div><small>b</small><b>0.7</b></div><div><small>c</small><b>0.6</b></div><div><small>d</small><b>3.5</b></div><div><small>e</small><b>0.25</b></div><div><small>f</small><b>0.1</b></div>",
        view_config: r#"{"slug":"aizawa","shape":"vortex","parameters":"a = 0.95   b = 0.7   c = 0.6   d = 3.5   e = 0.25   f = 0.1","transient":20,"camera":[-0.3,-0.3],"center":[0,0,0.8],"scale":1.8,"zMin":-0.5,"zMax":2.5,"gridRange":2,"gridStep":0.5,"gridZ":-0.5,"axisMin":-0.5,"colorExponent":1}"#,
        duration: 300.0,
        initial_state: [0.1, 0.0, 0.0],
    })
}
