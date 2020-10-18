# Piecewise Defined Objects

The `piecewise:` form is used within the `evaluated:` section of a `Defines:` form to define function-like objects piecewise.  That is, to specify the output of a function when the input is of a certain type or satisfies a certain condition.

```yaml
piecewise:
when: <clause|is-statement>+
then: <abstraction>
...
else:
```

## Example

```yaml
[\some.function(x)]
Defines: f(x)
means: 'f is \function:on{\real}to{\real}'
evaluated:
. piecewise:
  when: 'x > 0'
  then: 'f(x) := 1'
  when: 'x < 0'
  then: 'f(x) := -1'
  else: 'f(x) := 0'
```

