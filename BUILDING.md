# Building cauchy-ode

[Back to README](README.md)

## Windows (Intel Fortran)

Install the following x64 tools:

- Current stable Rust with the `x86_64-pc-windows-msvc` toolchain.
- Visual Studio 2022 or Build Tools with **Desktop development with C++**,
  including the MSVC x64 tools and a Windows SDK.
- Intel oneAPI **Fortran Compiler** (`ifx`) and **oneMKL** development libraries.

From PowerShell, Command Prompt, or an IDE terminal in the repository, use Cargo
directly:

```powershell
cargo build --release
cargo test
cargo run --release --example lorenz
```

The build script runs Intel's `setvars.bat` in a child process to find `ifx`,
the MSVC librarian, and the Intel runtime and oneMKL libraries. It passes the
detected library directories to Rust's linker, so `LIB` and `MKLROOT` need not
already be set in your terminal. This does not change your shell environment.
It uses `%ProgramFiles(x86)%\Intel\oneAPI` by default. Set `ONEAPI_ROOT` to the
oneAPI installation directory if installed elsewhere; an explicit `MKLROOT`
selects a different oneMKL installation.

The Intel compiler and oneMKL **runtime DLL directories** must still be on
`PATH` to run the built executables (normally `compiler\latest\bin` and
`mkl\latest\bin` under the oneAPI installation). If they are not on `PATH`,
the helper sets up the environment for both building and running:

```powershell
.\scripts\cargo-intel.cmd run --release --example lorenz
.\scripts\cargo-intel.cmd test
```

Alternatively, open an **Intel oneAPI command prompt for Intel 64** and run
Cargo directly. To initialize an ordinary **Command Prompt** manually:

```bat
call "C:\Program Files (x86)\Intel\oneAPI\setvars.bat" intel64
cargo build --release
cargo test
```

Windows builds default to `FC=ifx` and `AR=lib.exe`. `FC` may also name Intel
`ifort` or the full path to an Intel compiler; `AR` must use the MSVC librarian
command syntax. These variables name executables, without extra flags.
The supported Windows target is `x86_64-pc-windows-msvc`.

The build links the sequential LP64 oneMKL libraries, matching the Fortran
32-bit integer ABI and allowing concurrent Rust solves without MKL thread
pools. When distributing an executable, include the corresponding Intel
Fortran and oneMKL redistributable runtimes.

## Ubuntu Linux (ARM64 / aarch64)

The project builds and runs natively on Ubuntu 24.04 LTS ARM64 with GNU
Fortran and Ubuntu's BLAS/LAPACK libraries. Use current stable Rust from
[rustup](https://rustup.rs/) (the crate uses Rust edition 2024). On an ARM64
Ubuntu installation, rustup selects `aarch64-unknown-linux-gnu` automatically.

Install the native build dependencies, then build and test from the repository:

```bash
sudo apt update
sudo apt install build-essential gfortran libblas-dev liblapack-dev
cargo build --locked --release --all-targets
cargo test --locked
cargo test --locked --release
```

Run the console example or generate an interactive attractor viewer:

```bash
cargo run --locked --release --example hello_world
cargo run --locked --release --example lorenz -- --no-open
```

The second command writes `target/lorenz.html`, which you can open in a web
browser. On Ubuntu Desktop, omit `--no-open` to launch the browser through
`xdg-open` (provided by `xdg-utils`). Use `--no-open` on servers or over SSH.
This package is a library with runnable examples; select an example with
`--example` when using `cargo run`.

The build uses `gfortran` and `ar` from `PATH`. Set `FC` or `AR` to override
these executables; their values must be executable names or paths without
extra flags. Use the standard BLAS/LAPACK packages above, which have 32-bit
Fortran integers matching the Rust FFI even on ARM64. If copying executables
to another compatible Ubuntu ARM64 machine, install the runtime packages
`libgfortran5 libblas3 liblapack3` there as well.

The [Ubuntu ARM64 workflow](.github/workflows/ubuntu-arm64.yml) builds all
targets, runs debug and release tests, and runs the console and seven attractor examples on a native
`ubuntu-24.04-arm` runner. This covers 64-bit ARM; 32-bit ARM is not verified.

## Other Linux distributions and macOS (GNU Fortran)

Install `gfortran`, a C linker and archiver, and, on Linux, BLAS/LAPACK
development libraries using your distribution's package manager. On macOS,
install GCC with `brew install gcc`; Accelerate is supplied by macOS.
Then run `cargo build --release` and `cargo test`. Set `FC` if the GNU Fortran
executable has a versioned name or a custom path, and `AR` to override `ar`.
