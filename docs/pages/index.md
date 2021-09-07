---
layout: page
title: MathLingua
permalink: /
---

# Welcome to MathLingua

MathLingua is a language for easily creating a collection of mathematical
knowledge, including definitions, theorems, axioms, and conjectures, in a format
designed to be easy and fun to read and write.

Existing languages such as Lean, Coq, and others require their users to
have deep knowledge of their underlying frameworks (such as the Calculus of
Constructions) to express mathematical statements directly in those frameworks.

The goal of MathLingua, on the other hand, is to allow one to express
mathematical concepts in a higher level language that focuses on _what_ a
statement means instead of _how_ it is represented in a logical framework.

MathLingua then handles the details of how to express a statement in an
underlying logical framework.

This is similar to how mathematics is expressed in literature, where theorems
and definitions are described in a higher level natural language. For example,
the following illustrates how a Monoid is defined in MathLingua:

```python
[\monoid]
Defines: M := (X, *, e)
where:
. 'X is \set'
. '* is \function:on{X, X}to{X}'
. 'X in X'
means:
. [associativity]
  forAll: x, y, z
  where: 'x, y, z in X'
  then: 'x * (y * z) = (x * y) * z'
. [identity]
  forAll: x
  where: 'x in X'
  then:
  . 'e * x = x'
  . 'x * e = x'
```

Notice how the syntax of MathLingua allows one to think of a Monoid as a tuple
of objects with certain properties. Contrast this with how a Monoid could be
described in the Calculus of Constructions as a type consisting of fields of
different types (corresponding to the components of the Monoid) and proofs of
the various properties of those fields.

MathLingua's approach allows one to describe what a Monoid is while, under the
hood, MathLingua handles how to express a Monoid in a logical framework such as
the Calculus of Constructions.

**Note:** MathLingua is still a work in progress and is in the early stages of
development.

## Relationship to Existing Languages

Simply put, MathLingua is not designed to replace existing languages. Instead,
MathLingua is a way to more easily express mathematical knowledge that
transpiles down to existing languages such as Lean. Mathematicians can then
focus on _what_ a mathematical statement means instead of _how_ it is encoded
in a logical framework.

Compared to existing languages such that Lean, Coq, and others, MathLingua can
be viewed as a higher level language whereas those languages can be viewed as
lower level languages.

That is, Lean can be thought of as a lower level language similar to assembly
language in computer science whereas MathLingua can be viewed as a higher level
language language such as C, C++, or Java.

Then, just as in computer science as one can express an algorithm in both
assembly language and C, the meaning of the algorithm can be easier to
understand when written in C, whereas _how_ the algorithm is implemented in
particular hardware is more clear in assembly language.

Similarly, lower level languages such as Lean allow one to express _how_ a
mathematical concept is expressed in the Calculus of Constructions, MathLingua,
like C, helps makes the meaning of the concept more clear.

Lean, Coq, and others are great languages, but can have a steep learning curve
and require one to view mathematical knowledge from a type theory perspective.
MathLingua is designed to have a gradual learning curve that can be easily
used by mathematicians.

## Design Principles

MathLingua is being created with the following principles in mind:

- MathLingua provides a way to express Mathematical concepts in a way that is
  familiar to mathematicians without having to know advanced type theory.
- It is designed to have a gradual learning curve.
- It is focused on being easy to read and understand rather than expressing
  concepts with a small number of characters.
- Metadata is a core feature. All concepts can have descriptions of the
  importance of the concept and how it relates to other concepts as well as
  links to related books, articles, and other documents.
- Concepts are designed to be able to be read independently. You do not need
  to know what file a concept is defined in to understand it.
- One does not need to express mathematical knowledge directly in an framework
  such as the calculus of constructions.
- All types of concepts such as functions, sets, spaces, and others are
  represented consistently.

Note that these principles directly address some of the issues with existing
languages and frameworks such as Lean, Coq, Metamath, and others that have a
steep learning curve and multiple ways to express the same concept which can
make it difficult to learn, understand, and grow mathematical knowledge
expressed in those frameworks.

## Non Goals

MathLingua is not intended to replace existing languages such as Lean, Coq,
Metamath, or others. Instead, it is meant to be used with those languages by
transpiling to them and using them as the underlying formal framework.

## Examples

The following shows how to describe an even integer:

```python
[\even.integer]
Defines: n
where: 'n is \integer'
means:
. exists: k
  where: 'k is \integer'
  suchThat: 'n = 2*k'
```

A mathematical structure such as a group can be described as:

```python
[\group]
Defines: G := (X, *, e)
where:
. 'X is \set'
. '* is \function:on{X, X}to{X}'
. 'e in X'
means:
. forAll: x, y, z
  where: 'x, y, x in X'
  then: 'x * (y * z) = (x * y) * z'
. forAll: x
  where: 'x in X'
  then:
  . 'x * e = x'
  . 'e * x = x'
. forAll: x
  where: 'x in X'
  then:
  . exists: y
    where: 'y in X'
    suchThat:
    . 'x * y = e'
    . 'y * x = e'
```

The following shows how to express Fermat's Last Theorem:

```yaml
[\fermats.last.theorem]
Theorem:
given: n
where:
. 'n is \integer'
. 'n > 2'
then:
. not:
  . exists: x, y, z
    where: 'x, y, z is \integer'
    suchThat:
    . 'x \neq 0'
    . 'y \neq 0'
    . 'z \neq 0'
    . 'x^n + y^n = z^n'
```

A piecewise defined function, such as the Heaviside function can be described
as:

```python
[\heaviside(x)]
Defines: f(x)
where: 'f is \function:on{\reals}to{\reals}'
evaludated:
. piecewise:
  when: 'x < 0'
  then: 'f(x) := 0'
  when: 'x > 0'
  then: 'f(x) := 1'
  else: 'f(x) := 1/2'
```

The set of even integers can be expressed as:

```python
[\even.integers]
Defines: X
where: 'X is \set'
collects:
. given: x
  where: 'x is \integer'
  all: 'x'
  suchThat: 'x is \even'
```

## Roadmap

MathLingua started as a language for describing the structure of mathematical
statements to build a database of knowledge where one could search for
mathematical concepts based on their structure.

Since then, responding to valuable user feedback, MathLingua has moved to being
a high level language to help formally define a collection of mathematical
knowledge.

Currently, the MathLingua parser has been implemented in Kotlin, has unit and
smoke tests, and does some AST validation. However, more validation work needs
to be done. That is, parsing MathLingua source code into an AST for further
processing is in good shape.

However, work has just started on taking that AST and transpiling it to lower
level languages. The first target for such a transpilation is Lean.

Furthermore, since MathLingua initially started as a language for only
describing the structure of mathematical statements, much work is still needed
on developing the aspects of the language that allow proofs of theorems to
be encoded in MathLingua.

The rough project roadmap is as follows:

- Developing the infrastructure for transpiling MathLingua statements to Lean.
  This approach will use `sorry` for all proofs.
- Allowing proofs for theorems to be directly encoded in an underlying language
  while the statement of the theorem is in MathLingua.
- Developing the proof aspects of the MathLingua so that proves can be directly
  expressed in MathLingua.
- Translating existing theorems encoded in the underlying language into
  MathLingua and ensuring the proves remain valid.

_Note:_ MathLingua is a personal project that is being worked on in the author's
spare time.

## Helping Out

If you would like to help out, reach out to
<a href="mailto:DominicKramer@gmail.com">DominicKramer@gmail.com</a> or open a
pull request at <a href="https://github.com/DominicKramer/mathlingua">
https://github.com/DominicKramer/mathlingua</a>.

Assistance in trying the language, testing and exploring the best ways to
express MathLingua code in Lean, and writing documentation are some ways to
help, but any feedback or help is greatly appreciated.

## About the Author

MathLingua is being developed by Dr. Dominic Kramer a mathematician, computer
scientist, and senior software engineer. Dominic received his PhD in pure
mathematics from Iowa State University and his research interests include the
intersection of mathematics and computer science, in particular the foundations
of mathematics, type theory, and language design.

He is particularly interested in how the design and structure of languages
assists in the development of concepts in those languages and the tradeoffs
needed to make a language powerful yet easy to learn and master.

Dominic has industry experience in the useage of type systems in huge software
projects and deeply enjoys not just the beauty of mathematics, but helping
others learn explore and discover mathematics.

He hopes MathLingua will help many more people learn and discover this beauty
of mathematics.
