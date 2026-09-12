! Adapters for the MEXP and INV entry points used by the ODE routines.
! Error ranges distinguish matrix failures from step-tolerance failure (65).
module cauchy_ode_solver_linalg
  use, intrinsic :: iso_c_binding, only: c_int, c_double
  implicit none
  interface
    subroutine matrix_exp(n, a, tol, e, work, info) bind(C, name="cauchy_ode_matrix_exp")
      import c_int, c_double
      integer(c_int), value :: n
      real(c_double), intent(in) :: a(*)
      real(c_double), value :: tol
      real(c_double), intent(out) :: e(*), work(*)
      integer(c_int), intent(out) :: info
    end subroutine
    subroutine matrix_inverse(n, a, inverse, pivots, work, work_len, info) &
        bind(C, name="cauchy_ode_matrix_inverse")
      import c_int, c_double
      integer(c_int), value :: n, work_len
      real(c_double), intent(in) :: a(*)
      real(c_double), intent(out) :: inverse(*), work(*)
      integer(c_int), intent(out) :: pivots(*), info
    end subroutine
  end interface
end module

subroutine mexp(m, a, e, eps, ierr)
  use cauchy_ode_solver_linalg
  use, intrinsic :: ieee_arithmetic, only: ieee_is_finite
  implicit none
  integer(c_int), intent(in) :: m
  real(c_double), intent(in) :: a(m*m), eps
  real(c_double), intent(out) :: e(m*m)
  integer(c_int), intent(out) :: ierr
  real(c_double), allocatable :: work(:)
  integer :: allocation_status
  allocate(work(2*m*m), stat=allocation_status)
  if (allocation_status /= 0) then
    ierr = -1000
    return
  end if
  if (.not. all(ieee_is_finite(a))) then
    ierr = 1003
    return
  end if
  call matrix_exp(m, a, eps, e, work, ierr)
  if (ierr /= 0) then
    ierr = 1000 + abs(ierr)
  else if (.not. all(ieee_is_finite(e))) then
    ierr = 1003
  end if
end subroutine

subroutine inv(m, a, inverse, ierr)
  use cauchy_ode_solver_linalg
  use, intrinsic :: ieee_arithmetic, only: ieee_is_finite
  implicit none
  integer(c_int), intent(in) :: m
  real(c_double), intent(in) :: a(m*m)
  real(c_double), intent(out) :: inverse(m*m)
  integer(c_int), intent(out) :: ierr
  real(c_double), allocatable :: work(:)
  integer(c_int), allocatable :: pivots(:)
  integer :: allocation_status
  allocate(work(64*m), pivots(m), stat=allocation_status)
  if (allocation_status /= 0) then
    ierr = -1000
    return
  end if
  if (.not. all(ieee_is_finite(a))) then
    ierr = -2000
    return
  end if
  call matrix_inverse(m, a, inverse, pivots, work, 64*m, ierr)
  if (ierr /= 0) then
    ierr = 2000 + abs(ierr)
  else if (.not. all(ieee_is_finite(inverse))) then
    ierr = -2000
  end if
end subroutine
