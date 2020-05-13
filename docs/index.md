Mathlingua is a language designed to describe mathematical definitions, theorems, axioms, and conjectures in a way that:

* Prioritizes clarity with respect to the meaning of a statement
* Is easy for both computers and people to read and write
* Has the same level of mathematical rigour as found in mathematical texts
* Allows a collection of theorems and definitions written in Mathlingua to be easily built from the ground up from more basic definitions
* Allows for a range of rigour ranging from a LaTeX description of a result to a fully structured description with connections to dependent definitions

## Motivating Examples

### Definitions

The following illustrates how to encode the delta-epsilon definition of a continuous real valued function.  Note that the definition also includes metadata that shows how an assertion that a function is a real continuous function could appear in math literature. 

```yaml
[\real.continuous.function:on{A}]
Defines: f(x)
assuming: 'A is \set'
means:
. 'f is \real.function:on{A}'
. for: x0, epsilon
  where:
  . 'x0 \in A'
  . 'epsilon > 0'
  then:
  . exists: delta
    suchThat:
    . 'delta > 0'
    . for: x
      where:
      . 'x \in A'
      . '\abs{x - x0} < delta'
      then:
      . '\abs{f(x) - f(x0)} < epsilon'
Metadata:
. written: 'f \in C^0(\mathbb{R})'
```

### Theorems

Further, the following shows how to encode the result that a real-valued function is continuous where it is differentiable.  It makes reference to the definition `\real.differentiable.function:on{?}` that has been omitted for brevity.

```yaml
Result:
. for: f, A
  where:
  . 'f is \real.differential.function:on{A}'
  then:
  . 'f is \real.continuous.function:on{A}'
```

### Conjectures

The following illustrates how the Goldbach Conjecture could be encoded.  Note that the metadata of the conjecture also states the name of the conjecture.

```yaml
Conjecture:
. for: k
  where:
  . 'k is \even.integer'
  . 'k > 2'
  then:
  . exists: p, q
    suchThat:
    . 'p, q is \prime'
    . 'k = p + q'
Metadata:
. name: "Goldbach Conjecture"
```

### Less Formal Descriptions

The following illustrates how a theorem could be describe in a less structured way by describing it in LaTeX.  Note that the source material for the result is provided, and the result references the page number of that source.  In addition, tags are specified for categorization purposes.

```yaml
[AbstractAlgebraTheoryAndApplications]
Source:
. type: "Book"
. name: "Abstract Algebra Theory and Applications"
. author: "Thomas W. Judson"
. date: "August 15, 2014"
. url: "http://abstract.ups.edu/download/aata-20140815.pdf"
. offset: "10"
Metadata:
. tag:
  . "Abstract Algebra"
  . "Algebra"


ProtoResult:
. "All cyclic groups of infinite order are isomorphic to $\mathbb Z$."
Metadata:
. reference:
  . source: "@AbstractAlgebraTheoryAndApplications"
    page: "146"
. tag: "Abstract Algebra"
```

