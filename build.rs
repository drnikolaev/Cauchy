use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;

fn run(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("failed to {description}: {error}"));
    assert!(status.success(), "failed to {description}: {status}");
}

fn link_gnu_runtime(compiler: &OsStr, target_os: &str) {
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

fn intel_environment() -> HashMap<String, OsString> {
    // Run Intel's own tool discovery in a child process. This also locates the
    // MSVC librarian in ordinary terminals where only rustc can find MSVC.
    let variables = ["PATH", "LIB", "LIBPATH", "INCLUDE", "MKLROOT"];
    for name in variables
        .into_iter()
        .chain(["ONEAPI_ROOT", "ProgramFiles(x86)"])
    {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let mut environment: HashMap<_, _> = variables
        .into_iter()
        .filter_map(|name| env::var_os(name).map(|value| (name.to_owned(), value)))
        .collect();
    let root = env::var_os("ONEAPI_ROOT").map(PathBuf::from).or_else(|| {
        env::var_os("ProgramFiles(x86)")
            .map(|root| PathBuf::from(root).join("Intel").join("oneAPI"))
    });
    if let Some(setvars) = root
        .map(|root| root.join("setvars.bat"))
        .filter(|path| path.is_file())
    {
        println!("cargo:rerun-if-changed={}", setvars.display());
        let mut setup = Command::new("cmd.exe");
        // cmd.exe needs its own quoting, rather than MSVC argument escaping.
        // `set` emits UTF-16 with /u, including non-ASCII installation paths.
        setup.args(["/d", "/u", "/c"]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            setup.raw_arg("call \"%CAUCHY_ONEAPI_SETVARS%\" intel64 >nul && set");
        }
        let output = setup
            .env("CAUCHY_ONEAPI_SETVARS", &setvars)
            .env_remove("SETVARS_COMPLETED")
            .output()
            .expect("failed to initialize Intel oneAPI with cmd.exe");
        assert!(
            output.status.success(),
            "failed to initialize {}; install Intel Fortran, oneMKL, and the MSVC C++ tools, \
             or run scripts\\cargo-intel.cmd to see setup diagnostics",
            setvars.display()
        );
        let wide: Vec<_> = output
            .stdout
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        for line in String::from_utf16_lossy(&wide).lines() {
            if let Some((name, value)) = line.split_once('=') {
                let name = name.to_ascii_uppercase();
                if variables.contains(&name.as_str()) {
                    environment.insert(name, value.into());
                }
            }
        }
        // An explicitly selected MKL installation takes precedence over latest.
        if let Some(root) = env::var_os("MKLROOT") {
            environment.insert("MKLROOT".into(), root);
        }
    }
    environment
}

fn intel_tool(name: &OsStr, environment: &HashMap<String, OsString>) -> PathBuf {
    let executable = PathBuf::from(name);
    let with_extension = if executable.extension().is_none() {
        executable.with_extension("exe")
    } else {
        executable.clone()
    };
    for path in [executable.clone(), with_extension.clone()] {
        if path.is_file() {
            return path;
        }
    }
    if let Some(paths) = environment.get("PATH") {
        for directory in env::split_paths(paths) {
            for path in [directory.join(&executable), directory.join(&with_extension)] {
                if path.is_file() {
                    return path;
                }
            }
        }
    }
    panic!(
        "could not find {}; install Intel Fortran and the MSVC C++ tools, \
            or set ONEAPI_ROOT to your Intel oneAPI installation",
        executable.display()
    );
}

fn link_intel_runtime(environment: &HashMap<String, OsString>) {
    // rustc also discovers MSVC library paths itself, but it needs the Intel
    // paths from the oneAPI environment for the objects' /DEFAULTLIB records.
    let mut directories = Vec::new();
    if let Some(root) = environment.get("MKLROOT") {
        // oneAPI 2024+ uses lib; older releases use lib/intel64.
        for path in [
            PathBuf::from(root).join("lib"),
            PathBuf::from(root).join("lib/intel64"),
        ] {
            if path.is_dir() {
                directories.push(path);
            }
        }
    }
    if let Some(paths) = environment.get("LIB") {
        directories.extend(env::split_paths(paths).filter(|path| path.is_dir()));
    }
    for path in &directories {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    // Use the 32-bit-integer Fortran ABI (C_INT), and avoid an OpenMP runtime
    // or nested MKL thread pools when independent Rust solves run concurrently.
    for library in ["mkl_intel_lp64_dll", "mkl_sequential_dll", "mkl_core_dll"] {
        assert!(
            directories
                .iter()
                .any(|path| path.join(format!("{library}.lib")).is_file()),
            "could not find {library}.lib; install Intel oneMKL or set MKLROOT to its installation directory"
        );
        println!("cargo:rustc-link-lib=dylib={library}");
    }
}

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target OS must be set");
    let windows = target_os == "windows";
    if windows {
        assert_eq!(
            env::var("TARGET").unwrap(),
            "x86_64-pc-windows-msvc",
            "Intel Fortran on Windows requires the x86_64-pc-windows-msvc Rust target"
        );
    }
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set by Cargo"));
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let library = out_dir.join(if windows {
        "cauchy_ode_fortran.lib"
    } else {
        "libcauchy_ode_fortran.a"
    });
    let fortran_compiler =
        env::var_os("FC").unwrap_or_else(|| if windows { "ifx" } else { "gfortran" }.into());
    let archiver =
        env::var_os("AR").unwrap_or_else(|| if windows { "lib.exe" } else { "ar" }.into());
    let environment = if windows {
        intel_environment()
    } else {
        HashMap::new()
    };
    let fortran_compiler = if windows {
        intel_tool(&fortran_compiler, &environment).into_os_string()
    } else {
        fortran_compiler
    };
    let archiver = if windows {
        intel_tool(&archiver, &environment).into_os_string()
    } else {
        archiver
    };

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
    archive.envs(&environment);
    if windows {
        archive
            .arg("/nologo")
            .arg(format!("/OUT:{}", library.display()));
    } else {
        archive.arg("crs").arg(&library);
    }
    for source in sources {
        let path = manifest_dir.join("ftn").join(source);
        let object = out_dir
            .join(source)
            .with_extension(if windows { "obj" } else { "o" });
        let mut compile = Command::new(&fortran_compiler);
        compile.envs(&environment);
        if windows {
            let static_crt = env::var("CARGO_CFG_TARGET_FEATURE")
                .unwrap_or_default()
                .split(',')
                .any(|feature| feature == "crt-static");
            compile
                // Strict FP keeps NaN/Inf validation and adaptive error checks.
                .args([
                    "/nologo",
                    "/c",
                    "/O2",
                    "/recursive",
                    "/fp:strict",
                    "/threads",
                ])
                .arg(if static_crt {
                    "/libs:static"
                } else {
                    "/libs:dll"
                })
                .arg(format!("/module:{}", out_dir.display()))
                .arg(format!("/object:{}", object.display()));
        } else {
            compile
                .args(["-c", "-O2", "-fPIC", "-frecursive", "-J"])
                .arg(&out_dir)
                .arg("-I")
                .arg(&out_dir)
                .arg("-o")
                .arg(&object);
        }
        compile.arg(&path);
        run(&mut compile, &format!("compile {}", path.display()));
        archive.arg(object);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    run(&mut archive, "archive the Fortran objects");
    println!("cargo:rerun-if-env-changed=FC");
    println!("cargo:rerun-if-env-changed=AR");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=cauchy_ode_fortran");

    if windows {
        link_intel_runtime(&environment);
    } else {
        link_gnu_runtime(&fortran_compiler, &target_os);
        if target_os == "macos" {
            println!("cargo:rustc-link-lib=framework=Accelerate");
        } else {
            println!("cargo:rustc-link-lib=lapack");
            println!("cargo:rustc-link-lib=blas");
        }
    }
}
