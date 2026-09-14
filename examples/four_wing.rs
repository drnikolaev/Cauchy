// SPDX-License-Identifier: BSL-1.0
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE or https://www.boost.org/LICENSE_1_0.txt.

//! `cargo run --release --example four_wing`
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run(support::Demo {
        slug: "four_wing",
        system_name: "Four-wing",
        shape: "wings",
        index: "06",
        description: "Four wings from three quadratic equations.<br>Wide loops linked by narrow passages.",
        // This specific four-wing variant and parameter set:
        // https://analogparadigm.com/downloads/alpaca_45.pdf
        equations: ["0.2*x+y*z", "-0.01*x-0.4*y-x*z", "-z-x*y"],
        equation_html: "<div><span>ẋ</span> = 0.2x + yz</div><div><span>ẏ</span> = −0.01x − 0.4y − xz</div><div><span>ż</span> = −z − xy</div>",
        parameter_html: "<div><small>x growth</small><b>0.2</b></div><div><small>x → y</small><b>−0.01</b></div><div><small>y damping</small><b>0.4</b></div>",
        view_config: r#"{"slug":"four_wing","shape":"wings","parameters":"a = 0.2   b = -0.01   d = -0.4","transient":100,"camera":[-0.45,-0.35],"center":[0,0,0],"scale":3.3,"zMin":-3,"zMax":3,"gridRange":3,"gridStep":1,"gridZ":-3,"axisMin":-3,"colorExponent":1}"#,
        duration: 2000.0,
        initial_state: [0.1, 0.1, 0.1],
    })
}
