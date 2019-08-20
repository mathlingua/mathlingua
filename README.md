# MathLingua

> This is not an officially supported Google product.

MathLingua is a pair of languages, ChalkTalk and TexTalk, used to describe mathematical theorems, definitions, conjectures, and axiom in a way that can be easily written, read, and understood by both people and machines.

The following example shows how to encode one form of the Heine-Borel Theorem that describes that a subset of the reals is compact if and only if it is closed and bounded.
```yaml
Result:
. for: X
  where: 'X \subset \reals'
  then:
  . iff: 'X is \compact \set'
    then: 'X is \closed \bounded \set'
```
The structural parts of the result (written in an indented form with identifiers followed by a colon) are encoded in ChalkTalk.  The statements in single quotes that describes attributes of the mathematical objects in question are encoded in TexTalk.

ChalkTalk is designed to have a very small number of forms (that describe for all, exists, implication, etc.) that can describe mathematics while TexTalk is designed to qualify objects with a syntax very similar to LaTeX.

MathLingua (and its documentation) are still a work in progress.  More examples can be found in the [src/main/resources/mathlingua.txt](src/main/resources/mathlingua.txt) file.

## ChalkTalk

### Terminology

A complete unit of ChalkTalk text:
```yaml
Result:
. for: X
  where: 'X \subset \reals'
  then:
  . iff: 'X is \compact \set'
    then: 'X is \closed \bounded \set'
```
is called a **block**.  A block consists of **groups**.  In the example above, the text
```yaml
Result:
```
and
```yaml
for:
where:
then:
```
and
```yaml
iff:
then:
```
are each groups.  Last, each group consists of **sections**.  For exmaple, the group
```yaml
for:
where:
then:
```
has the sections
```yaml
for:
````
and
```yaml
where:
```
and
```yaml
then:
```
The sections of a group can have **arguments**.  For example, in the group
```yaml
iff: 'X is \compact \set'
then: 'X is \closed \bounded \set'
```
the section `if:` has the argument `'X is \compact \set'` and the section `then:` has the argument `'X is \closed \bounded \set'`.  Arguments, can also be written on a new line, with a leading period (.).  If an argument is a group, it must be written on a new line.

As an example, the group
```yaml
iff: 'X is \compact \set'
then: 'X is \closed \bounded \set'
```
could also be written as
```yaml
iff:
. 'X is \compact \set'
then:
. 'X is \closed \bounded \set'
```

Note that if a section has multiple arguments, they can be written on the same line of the section separated by commas, each on a new line with a preceding period, or a combination of both.  For example,
```yaml
for: x, y
. z
. w
then: 'x + y + z = w'
```

### Forms

ChalkTalk has the following forms.  In the syntax below, a section is suffixed by a question mark to indicate that the section is option.  If included, the section does not have a question mark.

The `<type>` synax is used to represent the type of the argument for a section where `<type>*` represents zero or more arguments of the given type, `<type>+` represents one or more arguments of the given type, and `<type1|type2|...|typeN>` is used to specify the argument is of type `type1` or `type2` or ... or `typeN`.  The `<|>` syntax can be used together with a `*` or `+` syntax.  A description of argument types is given in the next section.

For example, the form for the `for` group is:
```yaml
for: <name>+
where?: <clause>+
then: <clause>+
```
This means that each of the following are valid `for` group usages:
```yaml
for: x
where: 'x is \something'
then: '\somethingElse'
```
and
```yaml
for: x, y
then: '\something'
```

The following are all of the allowed forms in ChalkTalk:

#### Top Level Groups

```yaml
Result: <clause>+
```
```yaml
Conjecture: <clause>+
```
```yaml
Axiom: <clause>+
```
```yaml
[<statement>]
Defines: <target>
assuming?: <clause>+
means: <clause>+
```
```yaml
[<statement>]
Refines: <target>
assuming?: <clause>+
means: <clause>+
```
```yaml
Means: <clause>+
```

#### Structural Groups

```yaml
for: <target>+
where?: <clause>+
then: <clause>+
```
```yaml
exists: <target>+
suchThat: <clause>+
```
```yaml
if: <clause>+
then: <clause>+
```
```yaml
iff: <clause>+
then: <clause>+
```
```yaml
not: <clause>+
```
```yaml
or: <clause>+
```

### ChalkTalk Types

A ChalkTalk block must contain exactly one top level group.  Next, `clause` is an alias for
```yaml
statement|string|<structuralGroup>
```
and `target` is an alias for
```yaml
name|tuple|aggregate|abstraction|assignment
```

A **structuralGroup** is any of the Structural Groups listed above.

A **string** is of the form
```yaml
"<text>"
```
where `<text>` represents any sequence of UTF-8 encoded characters not including a double quotation mark.  Note that the text can contain newlines.

A **statement** is of the form
```yaml
'<text>'
```
where `<text>` represents any sequence of UTF-8 encoded characters not including a single quote.  The text in a statement is encoded in the TexTalk language, described later in this document.  Note that the text can contain newlines.

A **name** is sequence of characters maching the regex `[_a-zA-Z0-9]+` or the regex `[!@#$%^&*-+<>/\\|]+` and is used to specify a variable or an operator.

An **abstraction** is of the form
```yaml
<name>(<name>,+)
```
and is used to describe mappings such as functions, and functors, etc.  The following are examples of abstractions
```yaml
f(x)
f(x, y)
phi(a, b, c)
```

An **aggregate** is of the form
```yaml
{<name>,+}
```
and is used to represent a homogeneous collection of items (such as a set or class) where the name(s) are used to identify representative(s) from the collection.  The following are examples of aggregates
```yaml
{x}
{x, y}
```

An **assignment** is of the form
```yaml
<name> := <name|tuple|aggregate>
```
and is used to name a tuple or aggregate or rename a variable to treat each entity as a whole.  The following are examples of assignments.
```yaml
X := {x, y}
X := (x, y, z)
f := g
```

A **tuple** is of the form
```yaml
(<target>,+)
```
Note that by this definition, a tuple has a recursive structure.  Tuples are used to describe an entity that is a collection of a fixed finite number of elements (such as a group, ring, etc.).  The following are examples of tuples
```yaml
(X, +)
(X, F, *, +)
(A := (a, b, c := {x, y}), B := (C := (X, Y), (P, Q))
```
