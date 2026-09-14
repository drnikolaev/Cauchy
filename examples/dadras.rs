// SPDX-License-Identifier: BSL-1.0
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE or https://www.boost.org/LICENSE_1_0.txt.

//! `cargo run --release --example dadras`
mod support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run(support::Demo {
        slug: "dadras",
        system_name: "Dadras–Momeni",
        shape: "scrolls",
        index: "07",
        description: "Interwoven scrolls and sweeping sheets.<br>A three-scroll flow in constant motion.",
        // Dadras & Momeni (2009), doi:10.1016/j.physleta.2009.07.088.
        // https://el3ssar.github.io/TSDynamics/systems/ode/chaotic-attractors/Dadras/
        equations: ["y-3*x+2.7*y*z", "1.7*y-x*z+z", "2*x*y-9*z"],
        equation_html: "<div><span>ẋ</span> = y − ax + byz</div><div><span>ẏ</span> = cy − xz + z</div><div><span>ż</span> = dxy − ez</div>",
        parameter_html: "<div><small>a</small><b>3</b></div><div><small>b</small><b>2.7</b></div><div><small>c</small><b>1.7</b></div><div><small>d</small><b>2</b></div><div><small>e</small><b>9</b></div>",
        view_config: r#"{"slug":"dadras","shape":"scrolls","parameters":"a = 3   b = 2.7   c = 1.7   d = 2   e = 9","transient":20,"camera":[-0.4,-0.3],"center":[1,-1.5,-1],"scale":18,"zMin":-15,"zMax":12,"gridRange":15,"gridStep":5,"gridZ":-15,"axisMin":-15,"colorExponent":1}"#,
        duration: 300.0,
        initial_state: [2.5, 0.0, -1.5],
    })
}
