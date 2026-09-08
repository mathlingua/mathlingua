# Mathlingua

Mathlingua is a language for easily creating a collection of mathematical knowledge, including definitions, theorems,
axioms, and conjectures, in a format designed to be easy and fun to read and write. For more information, see
[mathlingua.org](http://www.mathlingua.org).

Start with the [language guide](docs/language.md) and the
[checked example collection](goldens/examples/). The precise references cover
[structural syntax](docs/structural_syntax.md),
[formulations](docs/formulation_syntax.md), and
[type and operator resolution](docs/type_and_operator_resolution.md).
Contributors can also read the [architecture guide](docs/architecture.md).
The author-facing [Mathlingua Reference](https://github.com/mathlingua/mathlingua-reference)
is maintained as a separate collection.

The `mlg` executable is self-contained. In particular, `mlg view` and
`mlg export` do not require Node.js or npm to be installed. Node.js is needed
only by contributors changing the viewer frontend under `web/`.
