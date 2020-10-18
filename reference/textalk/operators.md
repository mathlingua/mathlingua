# Operators

Operators either end with or contain only non-alpha-numeric characters (such as `+` or `\natural.+`, or `**`) or are infix commands (such as `\oplus` or `\set.union`).

A `using:` section is used to disambiguate operators that are ambiguous.  For example, in the theorem

```yaml
Theorem:
for: x
where: 'x is \natural'
then:
. exists: y
  where: 'y is \natural'
  suchThat: 'x + y = 0'
using:
. 'a + b := a \natural.+ b'
. '0 := \natural.0'
```

The `using:` section specifies that `+` means addition of natural numbers and `0` means the natural number `0`.

The `src/main/kotlin/mathlingua/common/textalk/PostProcessing.kt` file details the rules used to parse expressions involving operators unambiguously.