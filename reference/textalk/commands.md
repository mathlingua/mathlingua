# Commands

Commands are used to reference definitions.  In general commands contain three ways to specify parameters: in curly braces, in square braces, and in parentheses.

Curly braces are used to specify values that do non invoke the command as a function, parentheses specify values invoking the command as a function, and square braces introduce local variables.

For example, consider the command

```latex
\sin(x)
```

invokes `\sin` as a function with value `x` while the command

```latex
\continuous.function:on{A}to{B}
```

specifies a type of function.  That is, it is not a function invocation.  Last,

```latex
\integral[x]_{0}^{1}(f(x))
```

introduces the local variable `x` so that `f(x)` is the function `f` taking a single variable `x` as compared to the value of `f` at `x`.

The general form of a command is:

```latex
\f{a, b, c}.g.h[p,q]_{a}^{b}{x}:X{a, b}Y{c}(x, y)
```