# Definition

A `:=` statement is used to define the value of a variable or a function.

```latex
x := ...
f(x) := ...
(X, Y) := ...
```

A function invocation on the left side of a `:=` must contain only variables.  Nested invocations are not supported.  That is,

`f(x, y) := x + y` is valid, while `f(g(x, y), z) := x + y + z` is not.  For the second case, a `::=` statement must be used.  

Furthermore, the call `f(x)` on the left side of a `:=` is interpreted as defining `f` at the value `x`.  In contrast, `f(x)` on a non-colon-equals expression or on the right side of a `:=` is interpreted as the value of `f` at `x`.

To instead specify the function `f` that takes a single value `x`, one writes `f(x?)` or `f(?)` if naming the parameter is not needed.  Further, this allows specifying a function by partially applying another function.  That is,

```latex
f(x?, y + 2, z?)
```

specifyies an anonymous function with parameters `x` and `z` that would be defined explicitly as

```latex
g(x, z) := f(x, y + 2, z)
```