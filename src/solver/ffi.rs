//! Private, synchronous bridge to the Fortran step routines.
use super::{CallbackContext, CallbackKind, Method, Solver, SolverError};
use std::ffi::{c_double, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};

type Rhs = unsafe extern "C" fn(
    *const *mut c_void,
    *const c_int,
    *const c_double,
    *const c_double,
    *mut c_double,
    *mut c_int,
);
type Autonomous = unsafe extern "C" fn(
    *const *mut c_void,
    *const c_int,
    *const c_double,
    *mut c_double,
    *mut c_int,
);
type TimeOnly = unsafe extern "C" fn(
    *const *mut c_void,
    *const c_int,
    *const c_double,
    *mut c_double,
    *mut c_int,
);

macro_rules! declare_step {
    ($name:ident, $($arg:ident: $ty:ty),+) => {
        unsafe extern "C" {
            fn $name($($arg: $ty,)+ m: *const c_int,
                hmin: *const c_double, hmax: *const c_double,
                eps: *const c_double, p: *const c_double,
                x: *mut c_double, t: *mut c_double, h: *mut c_double,
                work: *mut c_double, info: *mut c_int);
        }
    };
}
declare_step!(cauchy_ode_sengl, context: *const *mut c_void, f: Rhs);
declare_step!(cauchy_ode_sloun, context: *const *mut c_void, f: Rhs, fj: Rhs);
declare_step!(cauchy_ode_sloui, context: *const *mut c_void, fa: TimeOnly, fi: TimeOnly);
declare_step!(cauchy_ode_slouu, context: *const *mut c_void, b: *const c_double, fu: Rhs);
declare_step!(cauchy_ode_srosn, context: *const *mut c_void, f: Rhs, fj: Rhs, ft: Rhs);
declare_step!(cauchy_ode_srosa, context: *const *mut c_void, f: Autonomous, fj: Autonomous);

/// All dimensions and controls have been checked by Solver before this call.
pub(super) fn step(
    solver: &Solver,
    context: &mut CallbackContext<'_, '_>,
    min_step: f64,
    state: &mut [f64],
    time: &mut f64,
    step: &mut f64,
    work: &mut [f64],
) -> c_int {
    let dimension = state.len() as c_int;
    context.current_time = *time;
    let matrix = context.system.constant_matrix.as_ptr();
    let context = (context as *mut CallbackContext<'_, '_>).cast::<c_void>();
    let mut info = 0;
    // SAFETY: Fortran retains no pointers. All arrays and the context outlive
    // the synchronous step; workspace bounds and integer sizes are checked by
    // the caller. Callbacks match solver_callbacks.f90. PC is a C_PTR passed by
    // reference as the first argument in every step routine and callback.
    macro_rules! call {
        ($name:ident, $($prefix:expr),+) => {
            unsafe { $name($($prefix,)+ &dimension, &min_step, &solver.max_step,
                &solver.tolerance, &solver.relative_threshold, state.as_mut_ptr(),
                time, step, work.as_mut_ptr(), &mut info) }
        };
    }
    match solver.method {
        Method::England => call!(cauchy_ode_sengl, &context, rhs),
        Method::Lawson => call!(cauchy_ode_sloun, &context, rhs, jacobian),
        Method::LawsonLinear => call!(cauchy_ode_sloui, &context, matrix_at_time, forcing),
        Method::LawsonSplit => call!(cauchy_ode_slouu, &context, matrix, remainder),
        Method::Rosenbrock => call!(cauchy_ode_srosn, &context, rhs, jacobian, time_derivative),
        Method::RosenbrockAutonomous => {
            call!(
                cauchy_ode_srosa,
                &context,
                autonomous_rhs,
                autonomous_jacobian
            )
        }
    }
    info
}

// Every pointer here comes from our private Fortran routines. X and output
// designate distinct live buffers; matrix output has M*M entries. Time-only
// callbacks use a null X pointer and never construct a slice from it.
unsafe fn evaluate(
    context: *mut c_void,
    m: *const c_int,
    time: f64,
    x: *const c_double,
    y: *mut c_double,
    info: *mut c_int,
    kind: CallbackKind,
) {
    let context = unsafe { &mut *context.cast::<CallbackContext<'_, '_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| {
        let m = unsafe { *m } as usize;
        let len = if matches!(kind, CallbackKind::Jacobian | CallbackKind::Matrix) {
            m * m
        } else {
            m
        };
        let state = if x.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(x, m) }
        };
        let output = unsafe { std::slice::from_raw_parts_mut(y, len) };
        context.evaluate(time, state, output, kind)
    }));
    context.error = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error),
        Err(_) => Some(SolverError::CallbackPanicked),
    };
    unsafe {
        *info = if context.error.is_some() { -1 } else { 0 };
    }
}

macro_rules! rhs_callback {
    ($name:ident, $kind:ident) => {
        unsafe extern "C" fn $name(
            pc: *const *mut c_void,
            m: *const c_int,
            t: *const c_double,
            x: *const c_double,
            y: *mut c_double,
            info: *mut c_int,
        ) {
            unsafe {
                evaluate(*pc, m, *t, x, y, info, CallbackKind::$kind);
            }
        }
    };
}
rhs_callback!(rhs, Rhs);
rhs_callback!(jacobian, Jacobian);
rhs_callback!(time_derivative, TimeDerivative);
rhs_callback!(remainder, Forcing);

macro_rules! time_callback {
    ($name:ident, $kind:ident) => {
        unsafe extern "C" fn $name(
            pc: *const *mut c_void,
            m: *const c_int,
            t: *const c_double,
            y: *mut c_double,
            info: *mut c_int,
        ) {
            unsafe {
                evaluate(*pc, m, *t, std::ptr::null(), y, info, CallbackKind::$kind);
            }
        }
    };
}
time_callback!(matrix_at_time, Matrix);
time_callback!(forcing, Forcing);

macro_rules! autonomous_callback {
    ($name:ident, $kind:ident) => {
        unsafe extern "C" fn $name(
            pc: *const *mut c_void,
            m: *const c_int,
            x: *const c_double,
            y: *mut c_double,
            info: *mut c_int,
        ) {
            let time = unsafe { (*(*pc).cast::<CallbackContext<'_, '_>>()).current_time };
            unsafe {
                evaluate(*pc, m, time, x, y, info, CallbackKind::$kind);
            }
        }
    };
}
autonomous_callback!(autonomous_rhs, Rhs);
autonomous_callback!(autonomous_jacobian, Jacobian);
