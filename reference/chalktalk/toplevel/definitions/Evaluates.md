# Disambiguation

The `Evaluates:` form is used to disambiguate operators.  For example, it can be used to specify that `a + b` means `a \natural.+ b` when `a` and `b` are both natural numbers.

```yaml
[<id>]
Evaluates:
when:
as:
...
using:
Metadata?: <metadata-form>
```

