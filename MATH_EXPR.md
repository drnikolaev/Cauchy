# MathExpr expression reference

`MathExpr` parses real-valued expressions, evaluates them with named variables,
and constructs symbolic derivatives. The same expression syntax is used by
`Solver` and all `OdeSystem` constructors, including matrix and forcing entries.
Function and variable names are case-sensitive.

## Operators and grouping

| Syntax | Meaning | Example |
| --- | --- | --- |
| `a + b` | Addition | `x+2` |
| `a - b` | Subtraction | `x-y` |
| `a * b` | Multiplication | `2*x` |
| `a / b` | Floating-point division | `1/2` evaluates to `0.5` |
| `a ^ b` | Power (floating-point `powf`) | `x^2`, `x^-2` |
| `-a` | Unary negation | `-x`, `--x` |
| `(a)` | Grouping | `(x+y)*z` |

From highest to lowest precedence: parentheses/function calls, powers, unary
negation, multiplication/division, addition/subtraction. Powers associate to
the right; the other binary operators associate to the left:

- `2^3^2` means `2^(3^2)` and evaluates to `512`.
- `-x^2` means `-(x^2)`; use `(-x)^2` to square a negative expression.
- `x^-2` means `x^(-2)`.
- `8/4/2` means `(8/4)/2` and evaluates to `1`.

Multiplication must be explicit: write `2*x` and `x*sin(t)`, not `2x` or
`x sin(t)`. Unary `+`, `**`, `%`, comparisons (`<`, `>=`, `==`, etc.), logical
operators, assignments, and the `?:` conditional operator are not supported.

## Functions

Every function below takes exactly one parenthesized expression, except `iif`,
which takes three comma-separated expressions. Calls can be nested.

| Function | Meaning |
| --- | --- |
| `sin(x)` | Sine; input in radians |
| `cos(x)` | Cosine; input in radians |
| `tan(x)` | Tangent; input in radians |
| `asin(x)` | Inverse sine; result in radians, real domain `-1 <= x <= 1` |
| `acos(x)` | Inverse cosine; result in radians, real domain `-1 <= x <= 1` |
| `atan(x)` | Inverse tangent; result in radians |
| `sinh(x)` | Hyperbolic sine |
| `cosh(x)` | Hyperbolic cosine |
| `tanh(x)` | Hyperbolic tangent |
| `sign(x)` | `-1` for negative values, `0` for either signed zero, `1` for positive values; NaN propagates |
| `abs(x)` | Absolute value |
| `exp(x)` | Exponential, e to the power x |
| `sqrt(x)` | Square root; real domain `x >= 0` |
| `log(x)` | Natural logarithm (base e); finite real values require `x > 0` |
| `log10(x)` | Base-10 logarithm; finite real values require `x > 0` |
| `iif(check, negative, otherwise)` | Evaluate `negative` when `check < 0`; otherwise evaluate `otherwise` |

There are no aliases such as `ln`, `arcsin`, or `sgn`, and no `pow`, `atan2`,
`min`, `max`, `floor`, or `ceil` functions. Use `x^y` for powers.

### Conditional expressions

`iif` uses the sign of its first argument, not a Boolean condition:

```text
iif(x, -x, x)                  # absolute value
iif(t-5, sin(t), cos(t))       # sin(t) before t=5, cos(t) at/after t=5
iif(x, sqrt(-x), sqrt(x))      # evaluate only the branch with a valid domain
```

The comments above explain the examples; `#` comments are not part of the
expression language. At zero, `iif` selects the third argument. A NaN check
also selects the third argument because the comparison with zero is false.
Only the selected branch is evaluated. However, `OdeSystem` validates variable
names in both branches when constructing the system, even if one is inactive.

## Numbers, names, and constants

Decimal and scientific notation are supported: `2`, `2.5`, `.5`, `2.`,
`1e-9`, and `2.5E+3`. Negative values use unary `-`. Whitespace between tokens
is ignored. Numeric separators such as `1_000` and hexadecimal literals are
not supported.

Identifiers start with a Unicode alphabetic character or `_`, followed by
Unicode alphanumeric characters or `_`. Examples: `x0`, `velocity`, `_rate`,
and `α`. Bare names are variables unless the numeric type recognizes them as
numeric literals. For `f64`, this includes `NaN` and `inf`; these names cannot
be declared as `OdeSystem` state, time, or parameter names.

There are no built-in `pi`, `e`, or `tau` constants: these are ordinary variable
names. Supply their values in the evaluation map, or bind them with an
`OdeSystem::*_with_parameters` constructor using `std::f64::consts`.
Variables such as `t`, `x`, and `x0` have no special meaning to standalone
`MathExpr`; the solver convenience methods establish their own naming rules.

## Evaluation and symbolic differentiation

```rust
use cauchy_ode::MathExpr;
use std::collections::HashMap;

fn main() {
    let expression = MathExpr::<f64>::parse("sin(pi/2)+x^2").unwrap();
    let values = HashMap::from([("pi", std::f64::consts::PI), ("x", 3.0)]);
    assert!((expression.evaluate(&values).unwrap() - 10.0).abs() < 1e-12);

    let derivative = expression.derive("x").simplify();
    assert!((derivative.evaluate(&values).unwrap() - 6.0).abs() < 1e-12);

    let branch = MathExpr::<f64>::parse("iif(x, sqrt(-x), sqrt(x))").unwrap();
    assert_eq!(branch.evaluate(&HashMap::from([("x", -4.0)])).unwrap(), 2.0);
    assert_eq!(MathExpr::<f64>::parse("2^3^2").unwrap()
                   .evaluate(&HashMap::new()).unwrap(), 512.0);
}
```

Parsing borrows variable names from the input string; keep that string alive
while using the expression. `ParseError` reports a byte offset and a message.
An undefined variable in an evaluated branch produces
`EvaluationError::VariableNotDefined`.

Evaluation follows floating-point arithmetic: division by zero, overflow, or
an out-of-domain function can return infinity or NaN inside `Ok`, rather than
an `EvaluationError`. For example, `sqrt(-1)` returns NaN. Negative bases with
fractional exponents are not general real-root operations; `(-8)^(1/3)` is not
a substitute for a cube-root function. Solver callbacks check evaluated values
for finiteness and return a `SolverError` when required values are non-finite.

`derive` supports every listed operation. For piecewise functions, it uses
`d sign(x)/dx = 0`, `d abs(x)/dx = sign(x)`, and differentiates only the two
result expressions of `iif`, preserving its check. These are branchwise rules,
not guarantees of differentiability at a switching point. Choose equations
whose derivatives exist where the selected solver needs them; `iif` does not
provide event detection or force a step to land on a discontinuity.

`simplify` applies real-algebra identities rather than strict IEEE-754
equivalences. Reassociation can change rounding or overflow, and eliminating
subexpressions can remove evaluation errors. `variables()` collects names from
all branches; `substitute()` replaces variables structurally, not textually.
