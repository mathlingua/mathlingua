# Foundational

A `Foundation:` form is used to specify something that the MathLingua translator provides.  For example, the translator could provide the structures and definitions of the natural numbers.

This is different from an `Axiom:` where the MathLingua translator assumes the statement is true.  With a `Foundation:` the translator creates the necessary definitions in the underlying language to support the definitions in the `Foundation:`.

```yaml
Foundation: <Defines:|States:|Views:|Mutually:>
Metadata?: <metadata-form>
```

## Example

```yaml
Foundation:
. [\natural]
  Defines: N
  means: "The set of natural numbers"
```

