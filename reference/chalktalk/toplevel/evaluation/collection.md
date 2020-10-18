# Set-Like Objects

The `collection:` form is used within the `evaluated:` section of a `Defines:` form to specify set-like objects.

```yaml
collection:
of: <target>+
in: <command-statement>
where: <clause>+
```

## Example

```yaml
[\primes]
Defines: P
means: 'P is \set'
evaluated:
. collection:
  of: p
  in: '\natural'
  where: 'p is \prime'
```

