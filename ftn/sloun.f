C     Lawson ODE numerical method step with precision control for systems x'=f(t,x)

C          Copyright Sergei Nikolaev 1992-2026
C Distributed under the Boost Software License, Version 1.0.
C    (See accompanying file LICENSE_1_0.txt or copy at
C          http://www.boost.org/LICENSE_1_0.txt)


C     F    - SUBROUTINE F(PC,M,T,X,FX,IERR) computes vector F(t,x)
C     FJ   - SUBROUTINE FJ(PC,M,T,X,XJ,IERR) computes Jacobi matrix FJ(t,x)
C     M    - [input] arrays' length, INTEGER
C     HMIN - [input] minimum step length allowed, DOUBLE PRECISION
C     HMAX - [input] maximum step length allowed, DOUBLE PRECISION
C     EPS  - [input] precision per step, DOUBLE PRECISION
C     P    - [input] absolute/relative precision per step threshold, DOUBLE PRECISION
C     X    - [input,output] source/result array, DOUBLE PRECISION
C     T    - [input,output] source/result time, DOUBLE PRECISION
C     H    - [input,output] RECOMMENDED step (actual step taken can be different), DOUBLE PRECISION
C     R    - [input,output] working array of 6*M^2+6*M DOUBLE PRECISION elements
C     IERR - [output] error code, zero for success, non-zero otherwise, INTEGER

      SUBROUTINE SLOUN (PC, F, FJ, 
     1                  M, HMIN, HMAX, EPS, P, X, T, H, R, IERR)
     & BIND(C, NAME="cauchy_ode_sloun")
      USE CAUCHY_ODE_CALLBACKS
CDEC$ IF DEFINED (FTN_EXPORTS)
CDEC$ ATTRIBUTES DLLEXPORT :: SLOUN
CDEC$ ENDIF
      DOUBLE PRECISION HMIN, HMAX, EPS, P, X, T, H, R
      TYPE(C_PTR) PC
      INTEGER   M, IERR
      DIMENSION R(*), X(M)
      PROCEDURE(rhs_callback) :: F, FJ

      DOUBLE PRECISION EPS1, THETA, AMPL, C1, ZERO /0.D0/, ONER /1.0D0/,
     1          CDS /31.0D0/, CRUNGE /0.0666666666666666666667D0/
      INTEGER   ONE /1/, MM, I1, I2, I3, I4, I5, I6, IDAMAX
      LOGICAL   BULD, BULHM, BULEX

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
      I6    = I5 + M * 6

      CALL FJ (PC, M, T, X, R, IERR)
      IF (IERR .NE. 0) RETURN
40    C1 = H / 4.D0
      CALL DCOPY (MM, R, ONE, R(I6), ONE)
      CALL DSCAL (MM, C1, R(I6), ONE)
      CALL MEXP  (M, R(I6), R(I1), EPS, IERR)
      IF (IERR .NE. 0) RETURN
      IF (.NOT. BULEX) THEN
         CALL DGEMM('N', 'N', M, M, M, ONER, R(I1), M, R(I1), M, 
     1                                           ZERO, R(I2), M)
      ENDIF
      C1 = H / 2.D0
      CALL SLOUNS (PC, F, M, X, T, H, R, R(I2), R(I5), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (M, R(I5), ONE, R(I3), ONE)
      CALL SLOUNS (PC, F, M, X, T, C1, R, R(I1), R(I5), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DCOPY  (M, R(I5), ONE, R(I4), ONE)
      CALL SLOUNS (PC, F, M, R(I4), T + C1, C1, R, R(I1), R(I5), IERR)
      IF (IERR .NE. 0) RETURN

      CALL DCOPY (M, R(I5), ONE, R(I4), ONE)
      CALL DSCAL (M,  CRUNGE, R(I4), ONE)
      CALL DAXPY (M, -CRUNGE, R(I3), ONE, R(I4), ONE)
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
      CALL DCOPY (MM, R(I1), ONE, R(I2), ONE)
      IF (DABS (H) .GT. DABS (HMIN)) GO TO 40
      H = DSIGN (HMIN, H)
      BULHM = .TRUE.
      BULEX = .FALSE.
      GO TO 40
110   IERR = 65
130   CONTINUE
      RETURN
      END



C     Lawson method step without precision control for systems x'=f(t,x) 
C     E should already have exp(A*h/2), where A=A(t+h/2)
C     R - working array of 6*M DOUBLE PRECISION elements
C     R has a solution when call is done

      SUBROUTINE SLOUNS (PC, F, M, X, T, H, A, E, R, IERR)
      USE CAUCHY_ODE_CALLBACKS
      DOUBLE PRECISION X, T, H, A, E, R
      TYPE(C_PTR) PC
      INTEGER   IERR
      INTEGER   M, I1, I2, I3, I4, I5
      DIMENSION R(*), A(1), E(1), X(M)
      PROCEDURE(rhs_callback) :: F

      INTEGER   ONE /1/
      DOUBLE PRECISION TT, ZERO /0.0D0/, ONER /1.0D0/, MONER /-1.0D0/
      DOUBLE PRECISION C1, C2, C3 /0.3333333333333333333333D0/, C4

      C1 = H / 2.D0
      C2 = C1 * C3
      C4 = H * C3
      TT = T + C1

      I1 = M  + ONE
      I2 = I1 + M
      I3 = I2 + M
      I4 = I3 + M
      I5 = I4 + M

      CALL F     (PC, M, T, X, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DGEMV ('N', M, M, MONER, A, M, X, ONE, ONER, R, ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, X, ONE, ZERO, R(I1), ONE)
      CALL DGEMV ('N', M, M, C1, E, M, R, ONE, ZERO, R(I2), ONE)
      CALL DCOPY (M, R(I1), ONE, R, ONE)
      CALL DAXPY (M, ONER, R(I2), ONE, R, ONE)
      CALL F     (PC, M, TT, R, R(I3), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DGEMV ('N', M, M, MONER, A, M, R, ONE, ONER, R(I3), ONE)
      CALL DCOPY (M, R(I1), ONE, R, ONE)
      CALL DAXPY (M, C1, R(I3), ONE, R, ONE)
      CALL F     (PC, M, TT, R, R(I4), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DGEMV ('N', M, M, MONER, A, M, R, ONE, ONER, R(I4), ONE)
      CALL DCOPY (M, R(I1), ONE, R, ONE)
      CALL DAXPY (M, H, R(I4), ONE, R, ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, R, ONE, ZERO, R(I5), ONE)
      CALL F     (PC, M, T + H, R(I5), R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL DGEMV ('N', M, M, MONER, A, M, R(I5), ONE, ONER, R, ONE)
      CALL DAXPY (M, C3, R(I2), ONE, R(I1), ONE)
      CALL DAXPY (M, C4, R(I3), ONE, R(I1), ONE)
      CALL DAXPY (M, C4, R(I4), ONE, R(I1), ONE)
      CALL DGEMV ('N', M, M, ONER, E, M, R(I1), ONE, C2, R, ONE)
      RETURN
      END
