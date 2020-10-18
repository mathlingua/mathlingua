# Is Statements

An `is` statement is used to describe the type and properties of an object.

```latex
x is \X
x, y is \X
x is \X \Y \Z
```

The right hand side of an `is` statement can contain a sequence of commands such as `x is \X \Y \Z`.  In this case, the `is` statement is interpreted as

```latex
x is \X.Z
x is \Y.Z
```

For example, one could concisely describe a set as a finite, compact, and convex (assuming the definitions for the commands below have been created) with

```latex
X is \finite \compact \convex \set
```

since the above is intepreted as

```latex
X is \finite.set
X is \compact.set
X is \convex.set
```

