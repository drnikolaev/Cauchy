C     Lawson ODE numerical method step with precision control for systems x' = Bx + u(t,x) with constant matrix B

C          Copyright Sergei Nikolaev 1992-2026
C Distributed under the Boost Software License, Version 1.0.
C    (See accompanying file LICENSE_1_0.txt or copy at
C          http://www.boost.org/LICENSE_1_0.txt)

C     B    - [input] matrix B of MxM DOUBLE PRECISION elements
C     FU   - SUBROUTINE FU(PC,M,T,X,U,IERR) computes vactor u(t,x)
C     M    - [input] arrays' length, INTEGER
C     HMIN - [input] minimum step length allowed, DOUBLE PRECISION
C     HMAX - [input] maximum step length allowed, DOUBLE PRECISION
C     EPS  - [input] precision per step, DOUBLE PRECISION
C     P    - [input] absolute/relative precision per step threshold, DOUBLE PRECISION
C     X    - [input,output] source/result array, DOUBLE PRECISION
C     T    - [input,output] source/result time, DOUBLE PRECISION
C     H    - [input,output] RECOMMENDED step (actual step taken can be different), DOUBLE PRECISION
C     R    - [input,output] working array of 5*M^2+6*M  DOUBLE PRECISION elements
C     IERR - [output] error code, zero for success, non-zero otherwise, INTEGER

      SUBROUTINE SLOUU (PC, B, FU, 
     1                  M, HMIN, HMAX, EPS, P, X, T, H, R, IERR)
     & BIND(C, NAME="cauchy_ode_slouu")
      USE CAUCHY_ODE_CALLBACKS
CDEC$ IF DEFINED (FTN_EXPORTS)
CDEC$ ATTRIBUTES DLLEXPORT :: SLOUU
CDEC$ ENDIF
      DOUBLE PRECISION B, HMIN, HMAX, EPS, P, X, T, H, R
      TYPE(C_PTR) PC
      INTEGER   M, IERR
      DIMENSION R(*), B(M, M), X(M)
      PROCEDURE(rhs_callback) :: FU

      INTEGER   ONE /1/, MM, I1, I2, I3, I4, I5, IDAMAX
      LOGICAL   BULD, BULHM, BULEX
      DOUBLE PRECISION EPS1, THETA, AMPL, C1, ONER /1.0D0/, 
     1          ZERO /0.0D0/, CDS /32.0D0/, 
     2          CRUNGE /0.0666666666666666666667D0/

      IERR  = 0
      BULD  = .TRUE.
      BULHM = .FALSE.
      BULEX = .FALSE.
      EPS1  = EPS / CDS

      MM    = M * M
      I1    = MM + ONE
      I2    = I1 + MM
      I3    = I2 + MM
      I4    = I3 + MM
      I5    = I4 + MM

30    C1 = H / 4.D0
      CALL DCOPY (MM, B, ONE, R(I5), ONE)
      CALL DSCAL (MM, C1, R(I5), ONE)
      CALL MEXP (M, R(I5), R, EPS, IERR)
      IF (IERR .NE. 0) RETURN
      IF (.NOT. BULEX) THEN
         CALL DGEMM('N','N', M, M, M, ONER, R, M, R, M, ZERO, R(I1), M)
      ENDIF
      C1 = H / 2.D0
      CALL SLOUUS (PC, FU, M, X, T, H, R(I1), R(I5), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (M, R(I5), ONE, R(I2), ONE)
      CALL SLOUUS (PC, FU, M, X, T, C1, R, R(I5), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (M, R(I5), ONE, R(I3), ONE)
      CALL SLOUUS (PC, FU, M, R(I3), T + C1, C1, R, R(I5), IERR)
      IF (IERR .NE. 0) RETURN

      CALL DCOPY (M, R(I5), ONE, R(I4), ONE)
      CALL DSCAL (M,  CRUNGE, R(I4), ONE)
      CALL DAXPY (M, -CRUNGE, R(I2), ONE, R(I4), ONE)
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
      BULEX = .TRUE.
      CALL DCOPY (MM, R, ONE, R(I1), ONE)
      IF (DABS (H) .GT. DABS (HMIN)) GO TO 30
      H = DSIGN (HMIN, H)
      BULHM = .TRUE.
      BULEX = .FALSE.
      GO TO 30
110   IERR = 65
130   CONTINUE
      RETURN
      END



C     Lawson method step without precision control for systems x'=Bx + u(t,x) 
C     E should already have exp(A*h/2), where A=A(t+h/2)
C     R - working array of 6*M DOUBLE PRECISION elements
C     R has a solution when call is done

      SUBROUTINE SLOUUS (PC, FU, M, X, T, H, E, R, IERR)
      USE CAUCHY_ODE_CALLBACKS
      DOUBLE PRECISION X, T, H, E, R
      TYPE(C_PTR) PC
      INTEGER   IERR
      INTEGER   M, I1, I2, I3, I4, I5
      DIMENSION R(*), E(M,M), X(M)
      PROCEDURE(rhs_callback) :: FU

      INTEGER   ONE /1/
      DOUBLE PRECISION TT, ZERO /0.D0/, ONER /1.D0/,
     1          C1, C2, C3 /0.3333333333333333333333D0/, C4

      C1 = H / 2.D0
      C2 = C1 * C3
      C4 = H * C3
      TT = T + C1

      I1 = M  + ONE
      I2 = I1 + M
      I3 = I2 + M
      I4 = I3 + M
      I5 = I4 + M

      CALL FU    (PC, M, T, X, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DGEMV ('N', M, M, C1, E, M, R, ONE, ZERO, R(I1), ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, X, ONE, ZERO, R(I2), ONE)
      CALL DCOPY (M, R(I2), ONE, R, ONE)
      CALL DAXPY (M, ONER, R(I1), ONE, R, ONE)
      CALL FU    (PC, M, TT, R, R(I3), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY (M, R(I2), ONE, R, ONE)
      CALL DAXPY (M, C1, R(I3), ONE, R, ONE)
      CALL FU    (PC, M, TT, R, R(I4), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY (M, R(I2), ONE, R, ONE)
      CALL DAXPY (M, H, R(I4), ONE, R, ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, R, ONE, ZERO, R(I5), ONE)
      CALL FU    (PC, M, T + H, R(I5), R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DAXPY (M, C3, R(I1), ONE, R(I2), ONE)
      CALL DAXPY (M, C4, R(I3), ONE, R(I2), ONE)
      CALL DAXPY (M, C4, R(I4), ONE, R(I2), ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, R(I2), ONE, C2, R, ONE)
      RETURN
      END
