# Type and Operator Resolution

This document describes the algorithms used by the semantic checker. It is
written as pseudocode rather than as a language tutorial. The corresponding
implementation is primarily in
[`src/backend/semantic/typecheck.rs`](../src/backend/semantic/typecheck.rs).

## Terms

The checker does not assign every symbol one closed, nominal type. It records
facts and asks whether those facts imply a required fact. The important fact
forms for type resolution are:

```text
Is(subject, type-key, type-signature)
RefinedIs(subject, refined-type-key, refined-signature,
          base-type-key, base-signature)
FunctionType(subject, input-specs, output-spec)
Spec(subject, operator, target)
InfixSpec(subject, signature, arguments, target)
MemberOf(subject, collection)
```

A **key** is the checker's canonical textual identity for a symbol or
expression. A **signature** identifies a declared type independently of the
type's actual arguments. For example, `\element:of{X}` is a type key whose
signature is `\element:of{}`.

`context` contains the symbols and facts currently in lexical scope, together
with substitutions introduced by definitions, aliases, destructuring, and
matched parameters. `registry` contains collection-wide declarations,
definition outputs, subtype rules, equivalences, views, disambiguations, and
provided-symbol capabilities.

## Determining a symbol's type

There are two related operations:

1. `KNOWN_FACTS` answers “what type facts should be reported for this symbol or
   expression?” It is used for recorded type information and hover output.
2. `HAS_TYPE` answers “can this symbol be used where this type signature is
   required?” It is used during checking and operator resolution.

Keeping these separate matters. A symbol can satisfy a supertype through an
`extends:` rule without the checker replacing its directly declared type, and a
refined type implies both its base type and its individual refinement facts.

### Canonicalizing a symbol

```text
function CANONICAL_KEY(symbol, context):
    if symbol is a stropped symbolic name (backtick +, for example):
        symbol := remove the backticks

    repeat until unchanged:
        symbol := apply context substitutions to symbol

    return symbol
```

Substitutions are equivalence edges. Their internal representative is chosen
deterministically; callers must not attach semantic meaning to the spelling of
that representative.

### Reporting known facts

```text
function KNOWN_FACTS(expression, context, registry):
    subject := EFFECTIVE_KEY(expression, context, registry)

    # Calls, members, commands, and operators may obtain facts from the result
    # of the declaration or capability to which they resolve.
    facts := RESULT_FACTS(expression, subject, context, registry)

    if facts is empty:
        normalized := CANONICAL_KEY(subject, context)
        facts := every fact in context.facts whose subject is either
                 subject or normalized

    if facts is empty and expression directly asserts a fact:
        return { "asserts " + FACT_FROM(expression) }

    if facts is empty and expression is statement-shaped, or resolves to a
       theorem-like command:
        return { Is(expression, \statement, \statement) }

    return SORT_AND_DEDUPLICATE(facts)
```

For a plain name, `RESULT_FACTS` is empty, so its reported type facts are the
facts attached to that name in the current context. For a composite expression,
`RESULT_FACTS` instantiates the output of the resolved function, command,
member, or provided operator with the call-site arguments.

### Testing whether a symbol has a required type

```text
function HAS_TYPE(subject, required-signature, context, registry,
                  allow-views = false):
    subject := CANONICAL_KEY(subject, context)

    candidates := facts in context.facts about subject
    candidates += instantiated outputs of a command, infix command, or direct
                  component named by subject
    candidates += direct views advertised by a defined or realized value

    for fact in candidates:
        if FACT_IMPLIES_TYPE(fact, subject, required-signature,
                             context, registry, allow-views, seen = {}):
            return true

    return false


function FACT_IMPLIES_TYPE(fact, subject, required-signature,
                           context, registry, allow-views, seen):
    fact := normalize fact with context substitutions

    if fact is in seen:
        return false
    add fact to seen

    if fact is Is(subject, _, required-signature):
        return true
    if fact is RefinedIs(subject, _, required-signature, _, _):
        return true
    if fact is Is(subject, _, actual-signature) and
       actual-signature is declared equivalent to required-signature:
        return true

    derived := requirements declared by the type named in fact
    derived += direct supertypes produced by extends rules
    derived += the base type and component refinements of a RefinedIs fact

    if allow-views:
        derived += facts obtained through view rules

    return any FACT_IMPLIES_TYPE(next, subject, required-signature,
                                 context, registry, allow-views, seen)
               for next in derived
```

The cycle guard is essential because requirements, extensions, refinements, and
views can form recursive paths. Operator selection calls `HAS_TYPE` with
`allow-views = false`, which disables recursive view conversion. The direct
view of a defined or realized value is still one of `HAS_TYPE`'s initial
candidates; other view relationships may help validate the selected operator's
requirements afterward, but do not select an overload.

Numeric literals have one more rule in general requirement proving: after
explicit and derived facts fail, an undeclared numeric spelling can receive the
matching collection-wide `Specify:` type. This is a fallback, not a fact
inserted unconditionally into every context.

Specification facts are reduced only on resolution paths that explicitly ask
for it. In particular, the plain-operator common-type fallback first expands
facts such as `y "in" M` into their implied `is` facts. The explicitly
single-owner forms `:+` and `+:` do not perform that preliminary expansion.

## Resolving a binary operator

The four spellings select different resolution strategies:

| Source form | Resolution source |
| --- | --- |
| `a + b` | a bound `+`, otherwise `Disambiguates`, otherwise a common owner type |
| `a +: b` | the type of `b` |
| `a :+ b` | the type of `a` |
| `a :+: b` | an owner type satisfied by both `a` and `b` |

The colon points at the owning operand: the colon in `a :+ b` is beside `a`,
and the colon in `a +: b` is beside `b`.

The same algorithm applies to the other symbolic operators and to the
colon-decorated named-operator forms. Plain named operators such as `a |plus| b`
are always application sugar for `plus(a, b)`.

### Shared preparation

```text
function PREPARE_OPERANDS(a, b, context, registry):
    prepared := clone context
    prepared += explicit cast facts contained in a and b
    prepared += inferred result facts for a
    prepared += inferred result facts for b

    actuals := [
        EFFECTIVE_KEY(a, prepared, registry),
        EFFECTIVE_KEY(b, prepared, registry),
    ]
    return (actuals, prepared)
```

`EFFECTIVE_KEY` recursively resolves calls, members, operators, and commands to
the key of their selected target. A recursion guard returns the raw normalized
key when resolution cycles.

### Plain `a + b`

```text
function RESOLVE_PLAIN_OPERATOR(a, "+", b, context, registry):
    if "+" is a bound symbol in the current lexical context:
        return RESOLVE_CALL("+", [a, b])

    (actuals, prepared) := PREPARE_OPERANDS(a, b, context, registry)

    if registry contains a Disambiguates entry for binary "+":
        for branch in source order:
            substitutions := bind branch parameters to actuals
            if every branch requirement is provable without views:
                return CHECK_TARGET(branch.to, substitutions, prepared)

        if the entry has else:
            return CHECK_TARGET(entry.else, parameter bindings, prepared)

        error "Could not disambiguate operator `+` for arguments ..."

    # This fallback exists only when no Disambiguates entry owns the operator.
    prepared := REDUCE_SPEC_AND_MEMBERSHIP_FACTS(prepared, registry)
    actuals := [
        EFFECTIVE_KEY(a, prepared, registry),
        EFFECTIVE_KEY(b, prepared, registry),
    ]
    return RESOLVE_PROVIDED_OPERATOR("+", BOTH, actuals, prepared, registry)
```

A plain symbolic operator therefore prefers a lexically bound callable. If the
symbol is not bound, a `Disambiguates:` entry owns resolution completely: its
first matching `when:` branch wins, followed by `else:`. Only the absence of
such an entry permits the common-owner capability fallback.

Plain `=` and `!=` are special. The checker first tries a common-owner provided
capability, but if none exists the expression is still accepted as a built-in
statement for arbitrary operand types.

### Left-owned `a :+ b`

```text
function RESOLVE_LEFT_OWNED(a, "+", b, context, registry):
    (actuals, prepared) := PREPARE_OPERANDS(a, b, context, registry)
    return RESOLVE_PROVIDED_OPERATOR("+", LEFT, actuals, prepared, registry)
```

This selects a provided `+` capability whose owner signature is satisfied by
`a`. The right operand need not have the owner's type, although the selected
capability's target and its invoked commands may impose requirements on it.

### Right-owned `a +: b`

```text
function RESOLVE_RIGHT_OWNED(a, "+", b, context, registry):
    (actuals, prepared) := PREPARE_OPERANDS(a, b, context, registry)
    return RESOLVE_PROVIDED_OPERATOR("+", RIGHT, actuals, prepared, registry)
```

This is the mirror image of `a :+ b`: `b` must satisfy the provided
capability's owner signature.

### Common-owned `a :+: b`

```text
function RESOLVE_COMMON_OWNED(a, "+", b, context, registry):
    (actuals, prepared) := PREPARE_OPERANDS(a, b, context, registry)
    return RESOLVE_PROVIDED_OPERATOR("+", BOTH, actuals, prepared, registry)
```

Both operands must satisfy the same provided capability's owner signature.
They may do so through direct types, refinements, extensions, or declared type
equivalence; their written type keys do not have to be identical.

The current implementation does not construct and rank a graph-theoretic least
common ancestor. It scans registered provided-symbol rules in registry order
and selects the first rule whose owner signature both operands satisfy.

### Selecting and applying a provided capability

```text
function RESOLVE_PROVIDED_OPERATOR(symbol, ownership, actuals,
                                   context, registry):
    key := BinaryOperator(symbol)

    for rule in registry.provided_symbols:
        if rule.key does not match key:
            continue
        if count(rule.parameters) != count(actuals):
            continue

        owner :=
            actuals[0]  if ownership = LEFT
            actuals[-1] if ownership = RIGHT
            actuals[0]  if ownership = BOTH

        owner-matches :=
            HAS_TYPE(actuals[0], rule.owner_signature) if ownership = LEFT
            HAS_TYPE(actuals[-1], rule.owner_signature) if ownership = RIGHT
            every HAS_TYPE(actual, rule.owner_signature) for actual in actuals
                if ownership = BOTH

        if not owner-matches:
            continue
        if rule requires a literal source and owner is not a collection literal:
            continue

        child := clone context
        bind rule.parameters positionally to actuals in child
        bind the owner's type parameters from owner's concrete type arguments
        bind rule.owner_subject to owner in child
        bind any destructured owner components in child

        CHECK_EXPRESSION(rule.target, child)
        return RESOLVED(rule.target, child)

    error "Could not resolve operator from the selected operand type"
```

The target is an ordinary expression, so checking it validates all command and
operator requirements after the formal operands and owner parameters have been
replaced by call-site values.

### Determining the resolved operator's output type

```text
function OPERATOR_RESULT_FACTS(operator-expression, result-subject,
                               context, registry):
    if operator-expression desugars to a bound function call:
        return instantiated output facts of that function call

    if a Disambiguates branch or else target was selected:
        return instantiated output facts obtained by resolving that target

    rule := the provided-symbol rule selected by the algorithm above
    if no rule was selected:
        return {}

    child := context with operand, owner, owner-type-parameter, and
             destructuring bindings applied
    facts := RESULT_FACTS(rule.target, result-subject, child, registry)
    return substitute the concrete call-site operands back into facts
```

Thus an operator does not have a separately guessed result type. Its result is
the result of the capability or disambiguation target to which the operator
resolves, instantiated with the actual operands.

## Failure summary

Resolution fails when any of the following holds:

- a referenced operand or bound operator symbol is out of scope;
- an explicitly owned form has no provided capability on the selected owner;
- a common-owned form has no rule whose owner signature both operands satisfy;
- a `Disambiguates:` entry has neither a matching `when:` branch nor `else:`;
- the selected target fails its own command, operator, or type requirements; or
- a resolution cycle reaches no independently known fact or target.
