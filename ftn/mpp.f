C          Copyright Sergei Nikolaev 1992-2026
C Distributed under the Boost Software License, Version 1.0.
C    (See accompanying file LICENSE_1_0.txt or copy at
C          http://www.boost.org/LICENSE_1_0.txt)

      SUBROUTINE MPP (M, B)
      INTEGER M, I
      DOUBLE PRECISION B, ONE /1.D0/
      DIMENSION B(M,M)
      DO 10 I=1,M
         B(I,I) = B(I,I) + ONE
10    CONTINUE
      RETURN
      END
