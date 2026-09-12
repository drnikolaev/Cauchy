C     England ODE numerical method step with precision control

C          Copyright Sergei Nikolaev 1992-2026
C Distributed under the Boost Software License, Version 1.0.
C    (See accompanying file LICENSE_1_0.txt or copy at
C          http://www.boost.org/LICENSE_1_0.txt)

C     F    - BIND(C) callback F(PC,M,T,X,Z,IERR) computes Z(T,X)
C     PC   - opaque caller-owned C pointer, passed by reference
C     Callback sets IERR to zero on success, negative on failure.
C     M    - [input] arrays' length, INTEGER
C     HMIN - [input] minimum step magnitude allowed, DOUBLE PRECISION
C     HMAX - [input] maximum step magnitude allowed, DOUBLE PRECISION
C     EPS  - [input] precision per step, DOUBLE PRECISION
C     P    - [input] absolute/relative precision threshold, DOUBLE
C     X    - [input,output] source/result array, DOUBLE PRECISION
C     T    - [input,output] source/result time, DOUBLE PRECISION
C     H    - [input,output] recommended step, DOUBLE PRECISION
C            (actual step taken can be different)
C     R    - [input,output] working array of 7*M DOUBLE elements
C     IERR - [output] error code, zero for success, non-zero otherwise, INTEGER

      SUBROUTINE SENGL (PC, F, M, HMIN, HMAX, EPS, P,
     &                  X, T, H, R, IERR)
     & BIND(C, NAME="cauchy_ode_sengl")
      USE CAUCHY_ODE_CALLBACKS
      IMPLICIT NONE
CDEC$ IF DEFINED (FTN_EXPORTS)
CDEC$ ATTRIBUTES DLLEXPORT :: SENGL
CDEC$ ENDIF
      TYPE(C_PTR) PC
      INTEGER(C_INT) M, IERR
      REAL(C_DOUBLE) HMIN, HMAX, EPS, P, X, T, H, R
      DIMENSION X(M), R(*)

      PROCEDURE(rhs_callback) :: F

      INTEGER   ONE /1/, I, M2, M3, M4, M5, M6, M7, MM7, IDAMAX
      DOUBLE PRECISION C, CDS /31.D0/, THETA, AMPL, EPS1, ZERO /0.D0/
      LOGICAL   BULD, BULHM, BULT
      DIMENSION C(21)

      DATA C
     1     /0.5D0, 0.6666666666666666666667D0, 0.2D0, 0.25D0, 2.0D0, 
     2      0.2592592592592592592593D0, 0.3703703703703703703704D0,
     3      0.0370370370370370370370D0, 0.0448D0, -0.2D0, 0.8736D0, 
     4      0.0864D0, -0.6048D0, -0.125D0, -0.6666666666666666666667D0, 
     5      -0.0625D0, 0.4821428571428571428571D0, 
     6      0.3720238095238095238095D0, 0.0416666666666666666667D0, 
     7      0.1041666666666666666667D0, -1.0D0/

      IERR  = 0
      EPS1  = EPS / CDS
      M2    = M  + 1
      M3    = M2 + M
      M4    = M3 + M
      M5    = M4 + M
      M6    = M5 + M
      M7    = M6 + M
      MM7   = M * 7
      
      BULT  = .TRUE.
      BULHM = .FALSE.
30    BULD  = .TRUE.

C TODO nullify it in calling code?
      DO 10 I = 1, MM7
          R(I) = ZERO
10    CONTINUE

      CALL F     (PC, M, T, X, R(M6), IERR)
      IF (IERR .NE. 0) GO TO 210
      CALL DAXPY (M, H, R(M6), ONE, R, ONE)
      CALL DCOPY (M, X, ONE, R(M7), ONE)
      CALL DAXPY (M, C(1), R, ONE, R(M7), ONE)
      CALL F     (PC, M, T + H*C(1), R(M7), R(M6), IERR)
      IF (IERR .NE. 0) GO TO 210
      CALL DAXPY (M, H, R(M6), ONE, R(M2), ONE)
      CALL DCOPY (M, X, ONE, R(M7), ONE)
      CALL DAXPY (M, C(4), R, ONE, R(M7), ONE)
      CALL DAXPY (M, C(4), R(M2), ONE, R(M7), ONE)
      CALL F     (PC, M, T + H*C(1), R(M7), R(M6), IERR)
      IF (IERR .NE. 0) GO TO 210
      CALL DAXPY (M, H, R(M6), ONE, R(M3), ONE)
      CALL DCOPY (M, X, ONE, R(M7), ONE)
      CALL DAXPY (M, C(21), R(M2), ONE, R(M7), ONE)
      CALL DAXPY (M, C(5), R(M3), ONE, R(M7), ONE)
      CALL F     (PC, M, T + H, R(M7), R(M6), IERR)
      IF (IERR .NE. 0) GO TO 210
      CALL DAXPY (M, H, R(M6), ONE, R(M4), ONE)
      CALL DCOPY (M, X, ONE, R(M7), ONE)
      CALL DAXPY (M, C(6), R, ONE, R(M7), ONE)
      CALL DAXPY (M, C(7), R(M2), ONE, R(M7), ONE)
      CALL DAXPY (M, C(8), R(M4), ONE, R(M7), ONE)
      CALL F     (PC, M, T + H*C(2), R(M7), R(M6), IERR)
      IF (IERR .NE. 0) GO TO 210
      CALL DAXPY (M, H, R(M6), ONE, R(M5), ONE)
      CALL DCOPY (M, X, ONE, R(M7), ONE)
      CALL DAXPY (M, C(9), R, ONE, R(M7), ONE)
      CALL DAXPY (M, C(10), R(M2), ONE, R(M7), ONE)
      CALL DAXPY (M, C(11), R(M3), ONE, R(M7), ONE)
      CALL DAXPY (M, C(12), R(M4), ONE, R(M7), ONE)
      CALL DAXPY (M, C(13), R(M5), ONE, R(M7), ONE)
      CALL F     (PC, M, T + H*C(3), R(M7), R(M6), IERR)
      IF (IERR .NE. 0) GO TO 210
      CALL DCOPY (M, R(M6), ONE, R(M2), ONE)
      CALL DSCAL (M, H, R(M2), ONE)

C     theta...
      CALL DCOPY (M, R, ONE, R(M7), ONE)
      CALL DSCAL (M, C(14), R(M7), ONE)
      CALL DAXPY (M, C(15), R(M3), ONE, R(M7), ONE)
      CALL DAXPY (M, C(16), R(M4), ONE, R(M7), ONE)
      CALL DAXPY (M, C(17), R(M5), ONE, R(M7), ONE)
      CALL DAXPY (M, C(18), R(M2), ONE, R(M7), ONE)

C     result...
      CALL DCOPY (M, X, ONE, R(M6), ONE)
      CALL DAXPY (M, C(19), R, ONE, R(M6), ONE)
      CALL DAXPY (M, C(20), R(M4), ONE, R(M6), ONE)
      CALL DAXPY (M, C(17), R(M5), ONE, R(M6), ONE)
      CALL DAXPY (M, C(18), R(M2), ONE, R(M6), ONE)

      THETA = DABS (R (IDAMAX (M, R(M7), ONE) + M7 - ONE))
      AMPL  = DABS (R (IDAMAX (M, R(M6), ONE) + M6 - ONE))
      IF (AMPL .GE. P) THETA = THETA / AMPL
      IF (THETA .GE. EPS) GO TO 160
      IF (BULT) GO TO 145
      BULD = .FALSE.
      GO TO 170
145   IF (THETA .GT. EPS1) BULD = .FALSE.
      GO TO 170
160   IF (BULHM) GO TO 200
      H = H * C(1)
      BULT = .FALSE.
      IF (DABS (H) .GT. DABS (HMIN)) GO TO 30
      H = DSIGN (HMIN, H)
      BULHM = .TRUE.
      GO TO 30
170   T = T + H
      CALL DCOPY (M, R(M6), ONE, X, ONE)
      IF (BULD) THEN
         H = H * C(5)
         IF (DABS (H) .GT. DABS (HMAX)) THEN
            H = DSIGN (HMAX, H)
         END IF
      END IF
      GO TO 210
200   IERR = 65
210   CONTINUE
      RETURN
      END
