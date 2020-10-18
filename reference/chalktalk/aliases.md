Aliases

```yaml
clause :=
| exists:
| for:
| if:
| iff:
| not:
| or:
```

```yaml
is-statement :=
| 'x... is X...'
| 'x is \X'
| 'x, y is \X'
| 'x is \X \Y \Z'
```

```yaml
using-clause :=
| 'f(x) := ...'
| '\f(x) := ...'
| 'X := ... '
| 'x + y := ...'
| Evaluates:
```

```yaml
abstraction :=
| f(x...)
| {f_n}_n
| {f(x)_n}_n
```

```yaml
target :=
| X
| (X, ...)
| <abstraction>
```

```yaml
colon-equals :=
| X := ...
| f(x...) := ...
```

```yaml
double-colon-equals :=
| f(x?, ...) ::= ...
| f(g(x?, y)) ::= ...
| f(g?(x?, y)) ::= ... 
```

```yaml
evaluated-clause :=
| collection:
| inductively:
| matching:
| piecewise:
| <colon-equals>
```

