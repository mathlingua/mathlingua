# Propositions

The `States:` form is used to describe propositions.  For example, it can be used to describe what it means for `n < m` where `n` and `m` are natural numbers.

```yaml
[<id>]
States:
when: <clause|is-statement>+
that: <clause|is-statement|abstraction>+
using: <using-clause>+
written: <text>
Metadata?: <metadata-form>
```

## Example

```yaml
[x \nat.< y]
States:
when: 'x, y is \nat'
that:
. '\nat.zero \nat.< y'
. if: 'x \nat.< y'
  then: 'x + 1 \nat.< y + 1'
using:
. 'x + y := x \nat.+ y'
```

