# Pattern Matching

The `matching:` form is used within the `evaluated:` section of a `Defines:` form to define function-like objects that are defined using pattern matching or matching against the base parts of a inductively defined object.

```yaml
matching: <double-colon-equals>+
```

## Example

```yaml
[\some.function(x)]
Defines: f(x)
means: 'f is \function:on{\natural}to{\natural}'
evaluated:
. matching:
  . 'f(\nat.0) ::= 1'
  . 'f(\nat.succ(x?)) ::= x - 1'
```

