C     Rosenbrock ODE numerical method with precision control for systems x'=f(t,x)

C          Copyright Sergei Nikolaev 1992-2026
C Distributed under the Boost Software License, Version 1.0.
C    (See accompanying file LICENSE_1_0.txt or copy at
C          http://www.boost.org/LICENSE_1_0.txt)

C     F    - SUBROUTINE F(PC,M,T,X,FX,IERR) computes vector f(t,x)
C     FJ   - SUBROUTINE FJ(PC,M,T,X,XJ,IERR) computes Jacobi matrix FJ(t,x)
C     FT   - SUBROUTINE FT(PC,M,T,X,XJ,IERR) computes vector of derivatives df(t,x)/dt
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

      SUBROUTINE SROSN (PC, F, FJ, FT,
     1                  M, HMIN, HMAX, EPS, P, X, T, H, R, IERR)
     & BIND(C, NAME="cauchy_ode_srosn")
      USE CAUCHY_ODE_CALLBACKS
CDEC$ IF DEFINED (FTN_EXPORTS)
CDEC$ ATTRIBUTES DLLEXPORT :: SROSN
CDEC$ ENDIF
      DOUBLE PRECISION HMIN, HMAX, EPS, P, X, T, H, R
      TYPE(C_PTR) PC
      INTEGER   M, IERR
      DIMENSION R(*), X(M)
      PROCEDURE(rhs_callback) :: F, FJ, FT

      INTEGER   ONE /1/, MM, I0, I1, I2, I3, I4, IT, IT2, IDAMAX
      LOGICAL   BULD, BULHM
      DOUBLE PRECISION EPS1, HS, THETA, AMPL,
     1          CDS /31.0D0/, CRUNGE /0.0666666666666666666667D0/

      IERR  = 0
      MM    = M * M
      IT    = MM + ONE
      IT2   = IT + M
      I0    = IT2 + M
      I1    = I0 + M
      I2    = I1 + M
      I3    = I2 + MM
      I4    = I3 + M
      BULD  = .TRUE.
      BULHM = .FALSE.
      EPS1  = EPS / CDS

      CALL FJ (PC, M, T, X, R, IERR)
      IF (IERR .NE. 0) RETURN
      CALL FT (PC, M, T, X, R(IT), IERR)
      IF (IERR .NE. 0) RETURN
18    CALL SROSNS (PC, F, M, X, T, R, R(IT), H, R(I0), IERR)
      IF (IERR .NE. 0) RETURN
19    HS = H
      H = H / 2.D0
      CALL SROSNS (PC, F, M, X, T, R, R(IT), H, R(I1), IERR)
      IF (IERR .NE. 0) RETURN
      CALL FJ (PC, M, T + H, R(I1), R(I2), IERR)
      IF (IERR .NE. 0) RETURN
      CALL FT (PC, M, T + H, R(I1), R(IT2), IERR)
      IF (IERR .NE. 0) RETURN
      CALL SROSNS (PC, F, M, R(I1), T + H, R(I2), R(IT2),
     1             H, R(I3), IERR)
      IF (IERR .NE. 0) RETURN
      H = HS

      CALL DCOPY (M, R(I3), ONE, R(I4), ONE)
      CALL DSCAL (M,  CRUNGE, R(I4), ONE)
      CALL DAXPY (M, -CRUNGE, R(I0), ONE, R(I4), ONE)
      THETA = DABS (R (I4 + IDAMAX (M, R(I4), ONE) - ONE))
      AMPL  = DABS (R (I3 + IDAMAX (M, R(I3), ONE) - ONE))
      IF (AMPL  .GE. P)    THETA = THETA / AMPL
      IF (THETA .GE. EPS1) BULD = .FALSE.
      IF (THETA .GT. EPS)  GO TO 100

      T = T + H
      CALL DCOPY (M, R(I3), ONE, X, ONE)
      IF (BULD) THEN
         H = H * 2.D0
         IF (DABS (H) .GT. DABS (HMAX)) THEN
            H = DSIGN (HMAX, H)
         END IF
      END IF
      GO TO 130
100   IF (BULHM) GO TO 110
      H = H / 2.D0
      IF (DABS (H) .GT. DABS (HMIN)) GO TO 105
      H = DSIGN (HMIN, H)
      BULHM = .TRUE.
      GO TO 18
105   CALL DCOPY (M, R(I1), ONE, R(I0), ONE)
      GO TO 19
110   IERR = 65
130   CONTINUE
      RETURN
      END


C     Rosenbrock method step without precision control for systems x'=f(t,x) 
C     A should already have the Jacobi matrix
C     AT should already have the derivatives vector
C     R - working array of 4*M^2+2*M DOUBLE PRECISION elements
C     R has a solution when call is done

C     SUBROUTINE INV (M, A, AI, IERR) - matrix inversion routine (external CVM)

      SUBROUTINE SROSNS (PC, F, M, X, T, A, AT, H, R, IERR)
      USE CAUCHY_ODE_CALLBACKS
      DOUBLE PRECISION X, T, A, AT, H, R, C
      TYPE(C_PTR) PC
      INTEGER   M, IERR, MM, I0, I1, I2, I3, I4
      DIMENSION R(*), A(M,M), AT(M), X(M), C(8)
      PROCEDURE(rhs_callback) :: F

      INTEGER   ONE /1/
      DOUBLE PRECISION ONER /1.0D0/, MONER /-1.0D0/, ZERO /0.0D0/

      DATA C
     1     /0.125D0, 0.375D0, 0.7916666666666666666667D0,
     2     -0.1666666666666666666667D0, 2.1666666666666666666667D0,
     3      0.1666666666666666666667D0, -2.0D0,
     4      0.6666666666666666666667D0/

      MM = M * M
      I0 = MM + ONE
      I1 = I0 + MM
      I2 = I1 + M
      I3 = I2 + MM
      I4 = I3 + MM

      CALL DCOPY (MM, A, ONE, R, ONE)
      CALL DSCAL (MM, -H, R, ONE)
      CALL MPP   (M, R)
      CALL INV   (M, R, R(I0), IERR)
      IF (IERR .NE. 0) RETURN
C     B matrix...
      CALL DSCAL (MM, H, R(I0), ONE)   
      CALL F     (PC, M, T, X, R(I1), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DAXPY (M, H, AT, ONE, R(I1), ONE)
      CALL DGEMV ('N', M, M, ONER, R(I0), M, R(I1), ONE, ZERO, R, ONE)
      CALL DCOPY (M, X, ONE, R(I3), ONE)
      CALL DAXPY (M, MONER, R, ONE, R(I3), ONE)
      CALL F     (PC, M, T - H, R(I3), R(I1), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DAXPY (M, H, AT, ONE, R(I1), ONE)
      CALL DGEMV ('N', M, M, ONER, R(I0), M, R(I1), ONE, 
     1             ZERO, R(I2), ONE)
      CALL DCOPY (M, X, ONE, R(I3), ONE)
      CALL DAXPY (M, C(1), R, ONE, R(I3), ONE)
      CALL DAXPY (M, C(2), R(I2), ONE, R(I3), ONE)
      CALL F     (PC, M, T + H / 2.D0, R(I3), R(I1), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DAXPY (M, H, AT, ONE, R(I1), ONE)
      CALL DGEMV ('N', M, M, ONER, R(I0), M, R(I1), ONE, 
     1             ZERO, R(I3), ONE)
      CALL DCOPY (M, X, ONE, R(I4), ONE)
      CALL DAXPY (M, C(2), R, ONE, R(I4), ONE)
      CALL DAXPY (M, C(3), R(I2), ONE, R(I4), ONE)
      CALL DAXPY (M, C(4), R(I3), ONE, R(I4), ONE)
      CALL F     (PC, M, T + H, R(I4), R(I1), IERR)
      IF (IERR .NE. 0) RETURN
      CALL DAXPY (M, H, AT, ONE, R(I1), ONE)
      CALL DGEMV ('N', M, M, ONER, R(I0), M, R(I1), ONE, 
     1             ZERO, R(I4), ONE)
      CALL DCOPY (M, X, ONE, R(I1), ONE)
      CALL DAXPY (M, C(5), R, ONE, R(I1), ONE)
      CALL DAXPY (M, C(6), R(I2), ONE, R(I1), ONE)
      CALL DAXPY (M, C(7), R(I3), ONE, R(I1), ONE)
      CALL DAXPY (M, C(8), R(I4), ONE, R(I1), ONE)
      CALL DCOPY (M, R(I1), ONE, R, ONE)
100   CONTINUE
      RETURN
      END
