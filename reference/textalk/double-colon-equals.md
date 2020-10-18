# Pattern Matching Definitions

A `::=` statement is used to define a function using pattern matching.  Such statements can only appear within a `using:` section or ` matching:` section.  On the left-hand side of a `::=` statement, any variable or function to match must have a `?` suffix.  Anything without such a suffix is matched literally.

```latex
f(x?, g(x?, y), ...) ::= ...
```

## Example

The following defines `f` where the second parameter needs to be an invocation of the function `g` whose second parameter is `w` while the third parameter of `f` must be `w`.

```latex
f(x?, g(y?, z), w) ::= x + y + z + w
```

In contrast, the following allows any function call as the second parameter, not only a call to `g`

```latex
f(x?, g?(y?, z), w) ::= x + y + z + w
```

