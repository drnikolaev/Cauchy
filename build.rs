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
        .filter(|_| path != PathBuf::from(library))
        .unwrap_or_else(|| panic!("Fortran compiler could not locate {library}"));

    println!("cargo:rustc-link-search=native={}", directory.display());
    println!("cargo:rustc-link-lib=gfortran");
}

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set by Cargo"));
    let daxpy_object = out_dir.join("daxpy.o");
    let evaluate_math_expr_object = out_dir.join("evaluate_math_expr.o");
    let matrix_exp_object = out_dir.join("matrix_exp.o");
    let matrix_inverse_object = out_dir.join("matrix_inverse.o");
    let library = out_dir.join("libcauchy_fortran.a");
    let fortran_compiler = env::var_os("FC").unwrap_or_else(|| "gfortran".into());
    let archiver = env::var_os("AR").unwrap_or_else(|| "ar".into());

    run(
        Command::new(&fortran_compiler)
            .args(["-c", "-O2", "-fPIC", "ftn/daxpy.f", "-o"])
            .arg(&daxpy_object),
        "compile ftn/daxpy.f",
    );
    run(
        Command::new(&fortran_compiler)
            .args(["-c", "-O2", "-fPIC", "ftn/evaluate_math_expr.f", "-o"])
            .arg(&evaluate_math_expr_object),
        "compile ftn/evaluate_math_expr.f",
    );
    run(
        Command::new(&fortran_compiler)
            .args(["-c", "-O2", "-fPIC", "ftn/matrix_exp.f", "-o"])
            .arg(&matrix_exp_object),
        "compile ftn/matrix_exp.f",
    );
    run(
        Command::new(&fortran_compiler)
            .args(["-c", "-O2", "-fPIC", "ftn/matrix_inverse.f", "-o"])
            .arg(&matrix_inverse_object),
        "compile ftn/matrix_inverse.f",
    );
    run(
        Command::new(archiver)
            .args(["crs"])
            .arg(&library)
            .arg(&daxpy_object)
            .arg(&evaluate_math_expr_object)
            .arg(&matrix_exp_object)
            .arg(&matrix_inverse_object),
        "archive the Fortran objects",
    );

    println!("cargo:rerun-if-changed=ftn/daxpy.f");
    println!("cargo:rerun-if-changed=ftn/evaluate_math_expr.f");
    println!("cargo:rerun-if-changed=ftn/matrix_exp.f");
    println!("cargo:rerun-if-changed=ftn/matrix_inverse.f");
    println!("cargo:rerun-if-env-changed=FC");
    println!("cargo:rerun-if-env-changed=AR");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=cauchy_fortran");

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target OS must be set");
    link_fortran_runtime(&fortran_compiler, &target_os);

    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=Accelerate");
    } else {
        println!("cargo:rustc-link-lib=lapack");
        println!("cargo:rustc-link-lib=blas");
    }
}
