      SUBROUTINE CAUCHY_MATRIX_EXP(N, A, TOL, E, WORK, INFO)
     &     BIND(C, NAME="cauchy_matrix_exp")
C     A, E, and the matrix workspaces use Fortran column-major order.
      USE, INTRINSIC :: ISO_C_BINDING, ONLY: C_INT, C_DOUBLE
      IMPLICIT NONE
      INTEGER(C_INT), VALUE, INTENT(IN) :: N
      REAL(C_DOUBLE), INTENT(IN) :: A(*)
      REAL(C_DOUBLE), VALUE, INTENT(IN) :: TOL
      REAL(C_DOUBLE), INTENT(OUT) :: E(*)
      REAL(C_DOUBLE), INTENT(OUT) :: WORK(*)
      INTEGER(C_INT), INTENT(OUT) :: INFO
      INTEGER(C_INT) :: I, J, K, NN, NSCALE
      INTEGER(C_INT), PARAMETER :: MAX_ITER = 1000
      REAL(C_DOUBLE) :: ALPHA, ANORM, ENORM, SCALE
      REAL(C_DOUBLE) :: TERMNORM, TARGET
      REAL(C_DOUBLE), PARAMETER :: ZERO = 0.0D0
      REAL(C_DOUBLE), PARAMETER :: ONE = 1.0D0
      REAL(C_DOUBLE) DLANGE
      EXTERNAL DAXPY, DCOPY, DGEMM, DLANGE

      INFO = 0
      IF (N .LT. 0) THEN
         INFO = -1
         RETURN
      END IF
      IF (TOL .LE. ZERO .OR. TOL .NE. TOL) THEN
         INFO = -2
         RETURN
      END IF
      IF (N .EQ. 0) RETURN

      NN = N * N
      DO 10 I = 1, NN
         E(I) = ZERO
         WORK(I) = ZERO
   10 CONTINUE
      DO 20 I = 1, N
         J = 1 + (I - 1) * (N + 1)
         E(J) = ONE
         WORK(J) = ONE
   20 CONTINUE

      ANORM = DLANGE('1', N, N, A, N, WORK(NN + 1))
      NSCALE = 0
      SCALE = ONE
   30 IF (ANORM / SCALE .LE. 0.5D0) GO TO 40
      IF (NSCALE .GE. 1023) THEN
         INFO = 2
         RETURN
      END IF
      NSCALE = NSCALE + 1
      SCALE = SCALE * 2.0D0
      GO TO 30

   40 TARGET = TOL / SCALE
      DO 50 K = 1, MAX_ITER
         ALPHA = ONE / (SCALE * DBLE(K))
         CALL DGEMM('N', 'N', N, N, N, ALPHA, WORK(1), N,
     &        A, N, ZERO, WORK(NN + 1), N)
         CALL DCOPY(NN, WORK(NN + 1), 1, WORK(1), 1)
         CALL DAXPY(NN, ONE, WORK(1), 1, E, 1)
         TERMNORM = DLANGE('1', N, N, WORK(1), N,
     &        WORK(NN + 1))
         ENORM = DLANGE('1', N, N, E, N, WORK(NN + 1))
         IF (TERMNORM .LE. TARGET * DMAX1(ONE, ENORM))
     &        GO TO 70
   50 CONTINUE
      INFO = 1

   70 IF (NSCALE .EQ. 0) RETURN
      CALL DGEMM('N', 'N', N, N, N, ONE, E, N, E, N,
     &     ZERO, WORK(NN + 1), N)
      CALL DCOPY(NN, WORK(NN + 1), 1, E, 1)
      NSCALE = NSCALE - 1
      GO TO 70
      END SUBROUTINE CAUCHY_MATRIX_EXP
