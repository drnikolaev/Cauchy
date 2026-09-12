! Explicit callback interfaces shared by the legacy solver routines.
! PC is an opaque C pointer passed by reference, never a 32-bit integer.
module cauchy_ode_callbacks
  use, intrinsic :: iso_c_binding, only: c_ptr, c_int, c_double
  implicit none
  abstract interface
    subroutine rhs_callback(pc, m, t, x, y, ierr) bind(C)
      import c_ptr, c_int, c_double
      type(c_ptr), intent(in) :: pc
      integer(c_int), intent(in) :: m
      real(c_double), intent(in) :: t, x(*)
      real(c_double), intent(out) :: y(*)
      integer(c_int), intent(out) :: ierr
    end subroutine
    subroutine autonomous_callback(pc, m, x, y, ierr) bind(C)
      import c_ptr, c_int, c_double
      type(c_ptr), intent(in) :: pc
      integer(c_int), intent(in) :: m
      real(c_double), intent(in) :: x(*)
      real(c_double), intent(out) :: y(*)
      integer(c_int), intent(out) :: ierr
    end subroutine
    subroutine time_callback(pc, m, t, y, ierr) bind(C)
      import c_ptr, c_int, c_double
      type(c_ptr), intent(in) :: pc
      integer(c_int), intent(in) :: m
      real(c_double), intent(in) :: t
      real(c_double), intent(out) :: y(*)
      integer(c_int), intent(out) :: ierr
    end subroutine
  end interface
end module
