# Mutual Definitions

The `Mutually:` form is used to specify definitions that depend on each other.  For example, a definition of an `odd` and `even` numbers can be defined inductively at the same time.

```yaml
Mutually: <Defines:>+
Metadata?: <metadata-form>
```

```yaml
Mutually: <States:>+
Metadata?: <metadata-form>
```

## Example

```yaml
Mutually:
. [\odd]
	Defines: Odd
	means: 'Odd is \Set'
	evaluated:
	. inductively:
	  from:
	  . constant: 0
	  . constructor: next(x)
	    on: '\odd'
. [\even]
  Defines: Even
  means: 'Even is \Set'
  evaluated:
  . inductively:
    from:
    . constant: 1
    . constructor: next(x)
      on: '\even'
```

