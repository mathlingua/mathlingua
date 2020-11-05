# MathLingua

MathLingua is a language for formally describing mathematical concepts
(definitions, theorems, axioms, and conjectures), with an emphasis on clearly
describing what the concept says and not how it is encoded in an underlying
type system or logical framework.

The goal of MathLingua is to create a language that can be used to build a
formal library of mathematical knowledge that doesn't have a steep learning
curve and is accessible to experts as well as those just starting to learn
advanced mathematics.

## Overview

MathLingua is not an automated theorem prover.  It is a language for describing
mathematical concepts.  This involves formally describing mathematical
definitions, theorems, axioms, and conjectures.  Proofs are currently outside
the scope of the project, but are part of the roadmap.

Further, MathLingua does not replace existing languages that are used to
formally describe mathematics such as [Lean](https://leanprover.github.io/),
[Coq](https://coq.inria.fr/), [Metamath](http://us.metamath.org/), or others.
Instead, it will be a layer on top of such languages with the goal to make it
easier to describe mathematics in those languages.
(Note that this is still a work in progress.)

In particular, MathLingua is designed to address the steep learning curves of
those languages as well as the need to be very comfortable with type theory
to properly use those languages.

That is, languages such as Lean are analogous to assembly languages is computer
science that describe **how** an algorithm should perform a computation, whereas
MathLingua is analogous to higher level languages like Java, Python, and others
that describe **what** an algorithm does.

For example, to describe a *group* in Lean, one needs to know what a group is
as well as **how** that description is encoded in the Calculus of Coinductive
constructions used by Lean.

With MathLingua, on the other hand, one only needs to focus on describing
**what** a group is.  The MathLingua compiler handles the task of determining
**how** to translate that description into an underlying formal language such
as Lean.

Moreover, MathLingua is designed to have a syntax that is easy to learn for
those just learning advanced mathematics as well as mathematics experts without
having to know advanced type theory.

For example, a group in MathLingua could be described as:
```yaml
[\group]
Defines: G := (X, *, e)
means:
. 'X is \set'
. '* is \function:on{X, X}to{X}'
. 'e \in X'
  [closure]
. for: x, y
  where: 'x, y \in X'
  then: 'x * y \in X'
. [associativity]
  for: x, y, z
  where: 'x, y, z \in X'
  then: '(x * y) * z = x * (y * z)'
. [identity]
  for: x
  where: 'x \in X'
  then: 'x * e = x'
. [inverse]
  for: x
  where: 'x \in X'
  then:
  . exists: y
    where: 'y \in X'
    suchThat: 'x * y = e'
```
The MathLingua compiler can then take that high level description and express it
in the language of Lean.  Note that the group definition is similar to how it
would be described in an abstract algebra textbook.

For more examples see the [src/test/resources/mathlingua.math](src/test/resources/mathlingua.math)
file.

## Status

MathLingua is still very much a work in progress.  Research into how the MathLingua
language should be designed to allow for formalism as well as clarity for those
without a type theory background is nearing completion.  Further, the MathLingua
parser has been implemented, and has a robust test suite.

However, the MathLingua documentation, including this document is still under
development.  In addition, the translation of MathLingua to an underlying formal
language is still in the design phase.

Check back often for updates, and if you would like to contribute contact
Dominic Kramer at *DominicKramer@gmail.com*, create an issue on this repo, or
open a pull request.  All contributions, suggestions, and insights are welcome.

> Note: MathLingua is not an officially supported Google product.

