      SUBROUTINE CAUCHY_ODE_MATRIX_INVERSE(N, A, AINV, IPIV, WORK,
     &     LWORK, INFO) BIND(C, NAME="cauchy_ode_matrix_inverse")
C     A and AINV use Fortran column-major order.
      USE, INTRINSIC :: ISO_C_BINDING, ONLY: C_INT, C_DOUBLE
      IMPLICIT NONE
      INTEGER(C_INT), VALUE, INTENT(IN) :: N, LWORK
      REAL(C_DOUBLE), INTENT(IN) :: A(*)
      REAL(C_DOUBLE), INTENT(OUT) :: AINV(*), WORK(*)
      INTEGER(C_INT), INTENT(OUT) :: IPIV(*), INFO
      INTEGER(C_INT) :: NN
      EXTERNAL DCOPY, DGETRF, DGETRI

      INFO = 0
      IF (N .LT. 0) THEN
         INFO = -1
         RETURN
      END IF
      IF (N .EQ. 0) RETURN
      IF (LWORK .LT. N) THEN
         INFO = -6
         RETURN
      END IF

      NN = N * N
      CALL DCOPY(NN, A, 1, AINV, 1)
      CALL DGETRF(N, N, AINV, N, IPIV, INFO)
      IF (INFO .NE. 0) RETURN
      CALL DGETRI(N, AINV, N, IPIV, WORK, LWORK, INFO)
      END SUBROUTINE CAUCHY_ODE_MATRIX_INVERSE
