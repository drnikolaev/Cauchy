C     Lawson ODE numerical method step with precision control for systems x' = A(t)x + fi(t) 

C          Copyright Sergei Nikolaev 1992-2026
C Distributed under the Boost Software License, Version 1.0.
C    (See accompanying file LICENSE_1_0.txt or copy at
C          http://www.boost.org/LICENSE_1_0.txt)

C     FA   - SUBROUTINE FA(PC,M,T,A,IERR) computes matrix A(t)
C     FI   - SUBROUTINE FI(PC,M,T,F,IERR) computes vector FI(t)
C     M    - [input] arrays' length, INTEGER
C     HMIN - [input] minimum step length allowed, DOUBLE PRECISION
C     HMAX - [input] maximum step length allowed, DOUBLE PRECISION
C     EPS  - [input] precision per step, DOUBLE PRECISION
C     P    - [input] absolute/relative precision per step threshold, DOUBLE PRECISION
C     X    - [input,output] source/result array, DOUBLE PRECISION
C     T    - [input,output] source/result time, DOUBLE PRECISION
C     H    - [input,output] RECOMMENDED step (actual step taken can be different), DOUBLE PRECISION
C     R    - [input,output] working array of 7*M^2+5*M DOUBLE PRECISION elements
C     IERR - [output] error code, zero for success, non-zero otherwise, INTEGER

      SUBROUTINE SLOUI (PC, FA, FI, 
     1                  M, HMIN, HMAX, EPS, P, X, T, H, R, IERR)
     & BIND(C, NAME="cauchy_ode_sloui")
      USE CAUCHY_ODE_CALLBACKS
CDEC$ IF DEFINED (FTN_EXPORTS)
CDEC$ ATTRIBUTES DLLEXPORT :: SLOUI
CDEC$ ENDIF
      DOUBLE PRECISION HMIN, HMAX, EPS, P, X, T, H, R
      TYPE(C_PTR) PC
      INTEGER   M, IERR
      DIMENSION R(*), X(M)
      PROCEDURE(time_callback) :: FA, FI

      INTEGER   ONE /1/, MM, I1, I2, I3, I4, I5, I6, I7, IDAMAX
      LOGICAL   BULD, BULHM
      DOUBLE PRECISION EPS1, THETA, AMPL, C1, C2, C3,
     1          CDS /31.0D0/, CRUNGE /0.0666666666666666666667D0/

      IERR  = 0
      BULD  = .TRUE.
      BULHM = .FALSE.
      EPS1  = EPS / CDS

      MM    = M * M
      I1    = MM + ONE
      I2    = I1 + MM
      I3    = I2 + MM
      I4    = I3 + MM
      I5    = I4 + MM
      I6    = I5 + MM + 4 * M
      I7    = I6 + M

30    C1 = H / 2.D0
      CALL FA     (PC, M, T + C1, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (MM, R, ONE, R(I1), ONE)
      CALL DSCAL  (MM, C1, R(I1), ONE)
      CALL MEXP   (M, R(I1), R(I2), EPS, IERR)
      IF (IERR .NE. 0) RETURN

      CALL SLOUIS (PC, FA, FI, M, X, T, H, R, R(I2), R(I5), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (M, R(I5), ONE, R(I6), ONE)

40    C1 = H  / 2.D0
      C2 = C1 / 2.D0
      C3 = C1 + C2

      CALL FA     (PC, M, T + C2, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (MM, R, ONE, R(I1), ONE)
      CALL DSCAL  (MM, C2, R(I1), ONE)
      CALL MEXP   (M, R(I1), R(I2), EPS, IERR)
      IF (IERR .NE. 0) RETURN
      CALL SLOUIS (PC, FA, FI, M, X, T, C1, R, R(I2), R(I5), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (M, R(I5), ONE, R(I7), ONE)

      CALL FA     (PC, M, T + C3, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (MM, R, ONE, R(I1), ONE)
      CALL DSCAL  (MM, C2, R(I1), ONE)
      CALL MEXP   (M, R(I1), R(I2), EPS, IERR)
      IF (IERR .NE. 0) RETURN
      CALL SLOUIS (PC, FA, FI, M, R(I7), T + C1, C1, R, R(I2),
     & R(I5), IERR)
      IF (IERR .NE. 0) RETURN

      CALL DCOPY (M, R(I5), ONE, R(I4), ONE)
      CALL DSCAL (M,  CRUNGE, R(I4), ONE)
      CALL DAXPY (M, -CRUNGE, R(I6), ONE, R(I4), ONE)

      THETA = DABS (R (I4 + IDAMAX (M, R(I4), ONE) - ONE))
      AMPL  = DABS (R (I5 + IDAMAX (M, R(I5), ONE) - ONE))
      IF (AMPL  .GE. P)    THETA = THETA / AMPL
      IF (THETA .GE. EPS1) BULD = .FALSE.
      IF (THETA .GT. EPS)  GO TO 100
      T = T + H      
      CALL DCOPY (M, R(I5), ONE, X, ONE)
      IF (BULD) THEN
         H = H * 2.D0
         IF (DABS (H) .GT. DABS (HMAX)) THEN
            H = DSIGN (HMAX, H)
         END IF
      END IF
      GO TO 130
100   IF (BULHM) GO TO 110
      H = H / 2.D0
      CALL DCOPY (M, R(I7), ONE, R(I6), ONE)
      IF (DABS (H) .GT. DABS (HMIN)) GO TO 40
      H = DSIGN (HMIN, H)
      BULHM = .TRUE.
      GO TO 30
110   IERR = 65
130   CONTINUE
      RETURN
      END


C     Lawson method step without precision control for systems x' = A(t)x + fi(t)
C     E should already have exp(A*h/2), where A=A(t+h/2)
C     R - working array of M^2 + 4*M DOUBLE PRECISION elements
C     R has a solution when call is done

      SUBROUTINE SLOUIS (PC, FA, FI, M, X, T, H, A, E, R, IERR)
      USE CAUCHY_ODE_CALLBACKS
      DOUBLE PRECISION X, T, H, A, E, R
      TYPE(C_PTR) PC
      INTEGER   IERR
      INTEGER   M, MM, I1, I2, I3, I4
      DIMENSION R(*), A(M,M), E(M,M), X(M)
      PROCEDURE(time_callback) :: FA, FI

      INTEGER   ONE /1/
      DOUBLE PRECISION ZERO /0.0D0/, ONER /1.0D0/, MONER /-1.0D0/,
     1          C1, C2, C3

      MM = M * M
      I1 = MM + ONE
      I2 = I1 + M
      I3 = I2 + M
      I4 = I3 + M

      C1 = H / 6.D0
      C2 = H / 2.D0
      C3 = (2.D0 * H) / 3.D0

      CALL FA    (PC, M, T, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DAXPY (MM, MONER, A, ONE, R, ONE)
      CALL FI    (PC, M, T, R(I1), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DGEMV ('N', M, M, ONER, R, M, X, ONE, ONER, R(I1), ONE)
      CALL FI    (PC, M, T + C2, R(I2), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY (M, R(I2), ONE, R(I3), ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, X, ONE, H, R(I3), ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, R(I3), ONE, ZERO, R(I4), ONE)
      CALL FA    (PC, M, T + H, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DAXPY (MM, MONER, A, ONE, R, ONE)
      CALL FI    (PC, M, T + H, R(I3), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DGEMV ('N', M, M, ONER, R, M, R(I4), ONE, ONER, R(I3), ONE)
      CALL DCOPY (M, X, ONE, R(I4), ONE)
      CALL DAXPY (M, C1, R(I1), ONE, R(I4), ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, R(I4), ONE, C3, R(I2), ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, R(I2), ONE, C1, R(I3), ONE)
      CALL DCOPY (M, R(I3), ONE, R, ONE)
      RETURN
      END
