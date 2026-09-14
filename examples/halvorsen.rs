// SPDX-License-Identifier: BSL-1.0
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE or https://www.boost.org/LICENSE_1_0.txt.

//! `cargo run --release --example halvorsen`
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run(support::Demo {
        slug: "halvorsen",
        system_name: "Halvorsen",
        shape: "trefoil",
        index: "05",
        description: "Three curling lobes in cyclic symmetry.<br>A triangular ribbon folded through space.",
        // Equations, parameters, and initial state:
        // https://juliadynamics.github.io/PredefinedDynamicalSystems.jl/stable/#PredefinedDynamicalSystems.halvorsen
        equations: [
            "-1.4*x-4*y-4*z-y^2",
            "-1.4*y-4*z-4*x-z^2",
            "-1.4*z-4*x-4*y-x^2",
        ],
        equation_html: "<div><span>ẋ</span> = −ax − 4y − 4z − y²</div><div><span>ẏ</span> = −ay − 4z − 4x − z²</div><div><span>ż</span> = −az − 4x − 4y − x²</div>",
        parameter_html: "<div><small>a · damping</small><b>1.4</b></div>",
        view_config: r#"{"slug":"halvorsen","shape":"trefoil","parameters":"a = 1.4","transient":20,"camera":[0.785,0.615],"center":[-2,-2,-2],"scale":12,"zMin":-15,"zMax":7,"gridRange":15,"gridStep":5,"gridZ":-15,"axisMin":-15,"colorExponent":1}"#,
        duration: 150.0,
        initial_state: [-8.6807408, -2.4741399, 0.070775762],
    })
}
