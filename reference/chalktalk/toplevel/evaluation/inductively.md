# Inductively Defined Objects

The `inductively:` form is used within the `evaluated:` section of a `Defines:` form to specify inductively defined objects.  That is, it is used to specify objects that are build "from the bottom up".

```yaml
inductively:
from:
. constant: <constant>+
. constructor: <abstraction>+
  on: <command-statement>+
```

## Example

```yaml
[\natural]
Defines: N
means: 'N is \set'
evaluated:
. inductively:
  from:
  . constant: 0
  . constructor: succ(x)
    on: '\natural'
```

