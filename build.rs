use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

fn run(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("failed to {description}: {error}"));
    assert!(status.success(), "failed to {description}: {status}");
}

fn link_fortran_runtime(compiler: &OsStr, target_os: &str) {
    let library = if target_os == "macos" {
        "libgfortran.dylib"
    } else {
        "libgfortran.so"
    };
    let output = Command::new(compiler)
        .arg(format!("-print-file-name={library}"))
        .output()
        .expect("failed to locate the Fortran runtime");
    assert!(output.status.success(), "failed to locate {library}");

    let path = PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("Fortran runtime path is not UTF-8")
            .trim(),
    );
    let directory = path
        .parent()
        .filter(|_| path != std::path::Path::new(library))
        .unwrap_or_else(|| panic!("Fortran compiler could not locate {library}"));

    println!("cargo:rustc-link-search=native={}", directory.display());
    println!("cargo:rustc-link-lib=gfortran");
}

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set by Cargo"));
    let library = out_dir.join("libcauchy_ode_fortran.a");
    let fortran_compiler = env::var_os("FC").unwrap_or_else(|| "gfortran".into());
    let archiver = env::var_os("AR").unwrap_or_else(|| "ar".into());

    // Compile the callback module before its users. Keep generated .mod files
    // in OUT_DIR, and make local work arrays private to concurrent solves.
    let sources = [
        "solver_callbacks.f90",
        "solver_linalg.f90",
        "evaluate_math_expr.f",
        "matrix_exp.f",
        "matrix_inverse.f",
        "mpp.f",
        "sengl.f",
        "sloun.f",
        "sloui.f",
        "slouu.f",
        "srosn.f",
        "srosa.f",
    ];
    let mut archive = Command::new(archiver);
    archive.arg("crs").arg(&library);
    for source in sources {
        let path = PathBuf::from("ftn").join(source);
        let object = out_dir.join(source).with_extension("o");
        run(
            Command::new(&fortran_compiler)
                .args(["-c", "-O2", "-fPIC", "-frecursive", "-J"])
                .arg(&out_dir)
                .arg("-I")
                .arg(&out_dir)
                .arg(&path)
                .arg("-o")
                .arg(&object),
            &format!("compile {}", path.display()),
        );
        archive.arg(object);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    run(&mut archive, "archive the Fortran objects");
    println!("cargo:rerun-if-env-changed=FC");
    println!("cargo:rerun-if-env-changed=AR");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=cauchy_ode_fortran");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target OS must be set");
    link_fortran_runtime(&fortran_compiler, &target_os);

    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=Accelerate");
    } else {
        println!("cargo:rustc-link-lib=lapack");
        println!("cargo:rustc-link-lib=blas");
    }
}
