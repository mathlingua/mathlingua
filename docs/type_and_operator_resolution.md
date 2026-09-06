# Type and Operator Resolution

This document describes the algorithms used by the semantic checker. It is an
implementation guide, written in typed pseudocode, rather than a language
tutorial. The corresponding implementation is primarily in
[`src/backend/semantic/typecheck.rs`](../src/backend/semantic/typecheck.rs),
with the data structures in
[`src/backend/semantic/types.rs`](../src/backend/semantic/types.rs).

The pseudocode uses these conventions:

- `List<T>` is ordered, `Set<T>` is unordered, and `Map<K, V>` is a key-value
  map.
- `Option<T>` is either `Some(T)` or `None`.
- `Result<T, E>` is either `Ok(T)` or `Err(E)`.
- `x := value` introduces or replaces a local variable.
- `return first ...` preserves the order of the list being searched.
- An `Expression` is an AST node. `RENDER(expression)` produces its canonical
  textual key; it is not user-facing pretty-printing.

## Data model and terms

The checker does not assign every expression one closed, nominal type. It
records facts and asks whether those facts imply a required fact.

```text
type Key = String
type TypeKey = Key
type TypeSignature = String
type OperatorName = String

enum FunctionSpec:
    IsSpec(type_key: TypeKey, signature: TypeSignature)
    SpecSpec(operator: OperatorName, target: Key)

enum TypeFact:
    Is(
        subject: Key,
        type_key: TypeKey,
        signature: TypeSignature,
    )
    RefinedIs(
        subject: Key,
        refined_type_key: TypeKey,
        refined_signature: TypeSignature,
        base_type_key: TypeKey,
        base_signature: TypeSignature,
    )
    FunctionType(
        subject: Key,
        inputs: List<FunctionSpec>,
        output: FunctionSpec,
        variadic_tuple_input: Boolean,
    )
    Spec(subject: Key, operator: OperatorName, target: Key)
    InfixSpec(
        subject: Key,
        signature: TypeSignature,
        arguments: List<Key>,
        target: Key,
    )
    MemberOf(subject: Key, collection: Key)
```

A **key** is the canonical textual identity of a symbol or expression. A
**type key** retains the concrete arguments of a type use. A **type signature**
identifies the declared type independently of those arguments.

For example, the declaration fact `x is \element:of{M}` is stored as:

```text
Is(
    subject = "x",
    type_key = "\element:of{M}",
    signature = "\element:of{}",
)
```

The distinction lets the checker answer both of these questions:

- Does `x` have the type signature `\element:of{}`?
- If so, which actual argument was supplied for the type parameter? Here it is
  `M`.

A refined fact stores both identities. For example, `x is
\(nonzero)::real` is represented conceptually as:

```text
RefinedIs(
    subject = "x",
    refined_type_key = "\nonzero::real",
    refined_signature = "\nonzero::real",
    base_type_key = "\real",
    base_signature = "\real",
)
```

It implies the base fact `x is \real` and the individual refinement facts
obtained by splitting a composite refinement. Thus a fact for
`\(commutative, associative)::operation` implies facts for both
`\(commutative)::operation` and `\(associative)::operation`.

The other fact forms retain the structure needed for later proof and
instantiation. Representative examples are:

| Source-level idea | Stored fact, omitting unimportant fields |
| --- | --- |
| `f is (_ is \real) -> (_ is \integer)` | `FunctionType("f", [IsSpec("\real", "\real")], IsSpec("\integer", "\integer"), false)` |
| `x "in" X` | `Spec("x", "in", "X")` |
| `H \:subgroup:/ G` | `InfixSpec("H", "\:subgroup:/", [], "G")` |
| `x member_of X` | `MemberOf("x", "X")` |

`Spec` preserves the quoted operator because its meaning is supplied by the
target's enabled specification rules. `MemberOf` represents the built-in
membership form. Both may later reduce to more specific facts, but they are not
eagerly replaced when first added to a context.

The two pieces of checker state used below are:

```text
record TypeContext:
    facts: List<TypeFact>
    substitutions: List<(Key, Key)>
    substitution_map: Map<Key, Key>
    symbols: Set<Key>
    defined_symbols: Set<Key>
    collection_literals: Map<Key, SetExpression>
    destructured_components: Map<Key, List<Key>>
    active_disambiguations: List<DisambiguationKey>
    defer_unresolved_provided_symbols: Boolean

record SignatureRegistry:
    type_infos: Map<TypeSignature, DefinitionTypeInfo>
    extension_rules: List<TypeExtensionRule>
    refinement_extension_rules: List<RefinementExtensionRule>
    equivalence_classes: List<EquivalenceClass>
    viewable_rules: List<ViewableRule>
    spec_rules: List<SpecOperatorRule>
    disambiguations: List<DisambiguationRule>
    provided_symbols: List<ProvidedSymbolRule>
    cast_as_rules: List<CastAsRule>
    numeric_specifications: NumericSpecifications

record DefinitionTypeInfo:
    signature: TypeSignature
    type_key: TypeKey
    parameters: List<Key>
    hidden_parameters: List<Key>
    using_parameters: List<Key>
    requirements: List<TypeFact>
    outputs: List<TypeFact>
    substitutions: List<(Key, Key)>
    defined_subject: Option<Key>
    described: Option<Key>
    component_types: List<TypeFact>
    component_refinements: List<TypeFact>
    parameter_destructurings: List<DestructuredParameter>

record TypeExtensionRule:
    subtype_signature: TypeSignature
    subject: Key
    parameters: List<Key>
    target: TypeFact

enum RefinementExtensionTarget:
    Fact(target: TypeFact)
    DynamicRefinedIs(subject: Key, command: RefinedCommandExpression)

record RefinementExtensionRule:
    subtype_signature: TypeSignature
    subject: Key
    parameters: List<Key>
    target: RefinementExtensionTarget

record ViewableRule:
    source_signature: TypeSignature
    source_subject: Key
    parameters: List<Key>
    target_subject: Key
    view_expression: Expression
    target: TypeFact

record SpecOperatorRule:
    owner_signature: TypeSignature
    owner_is_defined_value: Boolean
    owner_parameters: List<Key>
    source_subject: Option<Key>
    source_requires_literal: Boolean
    placeholder: Key
    operator: OperatorName
    target: Key
    kind: SpecOperatorAliasKind
    target_alias: SpecOperatorAliasTarget
    substitutions: List<(Key, Key)>

enum SpecOperatorAliasKind:
    Implies
    Iff

enum SpecOperatorAliasTarget:
    Conjunction(targets: List<SpecOperatorAliasTarget>)
    IsOrSpec(target: DeclarationRelation)
    MemberOf(collection: Expression)
    PlaceholderSpec(target: SpecificationExpression)
    Builtin(command: String)

record DisambiguationBranch:
    requirements: List<TypeFact>
    substitutions: List<(Key, Key)>
    target: Expression

record DisambiguationRule:
    key: DisambiguationKey
    parameters: List<Key>
    branches: List<DisambiguationBranch>
    else_expression: Option<Expression>

record ProvidedSymbolRule:
    owner_signature: TypeSignature
    owner_subject: Key
    source_subject: Option<Key>
    key: DisambiguationKey
    parameters: List<Key>
    target: Expression

record CastAsRule:
    owner_signature: TypeSignature
    owner_subject: Key
    source_subject: Key
    left: Expression
    right: Expression

record NumericTypeSpecification:
    type_key: TypeKey
    signature: TypeSignature

record NumericSpecifications:
    decimal: Option<NumericTypeSpecification>
    zero_or_positive_integer: Option<NumericTypeSpecification>
    positive_integer: Option<NumericTypeSpecification>
    integer: Option<NumericTypeSpecification>
```

`TypeContext` contains facts and symbols in the current lexical scope.
Substitutions come from definitions, aliases, destructuring, and parameter
bindings. `SignatureRegistry` contains collection-wide declaration metadata.
Lists of rules are in registration order, which is semantically relevant when
the checker selects the first matching rule.

For example, inside a theorem with:

```text
given:
. M ::= (X, *) is \magma
. x "in" M
```

the context contains a fact about `M`, a specification fact about `x`, and a
destructuring entry mapping `M` to `[X, *]`. The registry, rather than the local
context, contains the declaration of `\magma` and the capabilities it enables.

## Normalizing and resolving expression keys

### Normalizing a key

Backtick stropping lets an operator be referred to as a name. It does not
create a different symbol. Substitutions are equivalence edges, and their
internal representative is selected deterministically. Callers must not attach
semantic meaning to the representative's spelling.

```text
function UNSTROP_NAME(name: String) -> String:
    if name starts and ends with "`":
        return name with the first and last "`" removed
    return name

function NORMALIZE_KEY(key: Key, context: TypeContext) -> Key:
    result: Key := key

    # At most one new edge can be traversed per pass. The extra pass detects
    # convergence without allowing a malformed substitution cycle to loop.
    repeat at most LENGTH(context.substitutions) + 1 times:
        next: Key := SUBSTITUTE_KEY(result, context.substitution_map)
        if next = result:
            break
        result := next

    return result

function KEY_FOR_NAME(name: String, context: TypeContext) -> Key:
    return NORMALIZE_KEY(UNSTROP_NAME(name), context)

function NORMALIZE_FUNCTION_SPEC(
    spec: FunctionSpec,
    context: TypeContext,
) -> FunctionSpec:
    return match spec:
        IsSpec(type_key, signature) =>
            IsSpec(NORMALIZE_KEY(type_key, context), signature)
        SpecSpec(operator, target) =>
            SpecSpec(operator, NORMALIZE_KEY(target, context))

function NORMALIZE_FACT(fact: TypeFact, context: TypeContext) -> TypeFact:
    return match fact:
        Is(subject, type_key, signature) => Is(
            NORMALIZE_KEY(subject, context),
            NORMALIZE_KEY(type_key, context),
            signature,
        )
        RefinedIs(subject, refined_key, refined_signature,
                  base_key, base_signature) => RefinedIs(
            NORMALIZE_KEY(subject, context),
            NORMALIZE_KEY(refined_key, context),
            refined_signature,
            NORMALIZE_KEY(base_key, context),
            base_signature,
        )
        FunctionType(subject, inputs, output, variadic) => FunctionType(
            NORMALIZE_KEY(subject, context),
            MAP(spec => NORMALIZE_FUNCTION_SPEC(spec, context), inputs),
            NORMALIZE_FUNCTION_SPEC(output, context),
            variadic,
        )
        Spec(subject, operator, target) => Spec(
            NORMALIZE_KEY(subject, context),
            operator,
            NORMALIZE_KEY(target, context),
        )
        InfixSpec(subject, signature, arguments, target) => InfixSpec(
            NORMALIZE_KEY(subject, context),
            signature,
            MAP(key => NORMALIZE_KEY(key, context), arguments),
            NORMALIZE_KEY(target, context),
        )
        MemberOf(subject, collection) => MemberOf(
            NORMALIZE_KEY(subject, context),
            NORMALIZE_KEY(collection, context),
        )
```

Example: if the context records the equivalence `A = B`, then
`NORMALIZE_KEY("f(A)", context)` and `NORMALIZE_KEY("f(B)", context)` return the
same key. Consequently `Is("x", "\element:of{A}", "\element:of{}")` and the
same fact written with `B` normalize identically. Signatures and operator names
are not substituted. Likewise, ``KEY_FOR_NAME("`+`", context)`` refers to `+`.

### Finding an expression's effective key

The raw key describes the written expression. The **effective key** describes
the target selected for a call, member, command, or operator. This is important
because facts are usually declared on the selected target.

```text
function EFFECTIVE_KEY(
    expression: Expression,
    context: TypeContext,
    registry: SignatureRegistry,
) -> Key:
    resolving: Set<Key> := {}
    return EFFECTIVE_KEY_INNER(expression, context, registry, resolving)

function EFFECTIVE_KEY_INNER(
    expression: Expression,
    context: TypeContext,
    registry: SignatureRegistry,
    resolving: Set<Key>,
) -> Key:
    if expression is Grouped(inner) or Labeled(inner):
        return EFFECTIVE_KEY_INNER(inner, context, registry, resolving)

    raw: Key := NORMALIZE_KEY(RENDER(expression), context)
    if raw in resolving:
        return raw
    add raw to resolving

    selected: Option<Key> := match expression:
        FunctionCall(name, arguments) =>
            EFFECTIVE_FUNCTION_TARGET(name, arguments, context, registry,
                                      resolving)

        MemberCall(owner, name, arguments) =>
            EFFECTIVE_MEMBER_TARGET(owner, name, arguments, context, registry,
                                    resolving)

        MemberAccess(owner, name) =>
            EFFECTIVE_MEMBER_TARGET(owner, name, [], context, registry,
                                    resolving)

        Prefix(operator, operand) =>
            EFFECTIVE_PREFIX_TARGET(operator, operand, context, registry,
                                    resolving)

        Postfix(operand, named_operator) =>
            EFFECTIVE_KEY_INNER(
                DESUGAR_AS_CALL(named_operator, [operand]),
                context, registry, resolving,
            )

        Binary(left, operator, right) =>
            EFFECTIVE_BINARY_TARGET(left, operator, right, context, registry,
                                    resolving)

        Build(_, value) =>
            Some(EFFECTIVE_KEY_INNER(value, context, registry, resolving))

        otherwise => None

    remove raw from resolving
    return selected.unwrap_or(raw)
```

The specialized target functions use the same precedence described later in
this document: application sugar first, then the relevant provided-symbol or
disambiguation rule. Member lookup additionally checks direct destructured
components. If no target resolves, this function returns the normalized raw key;
the checking path is responsible for emitting a diagnostic.

Example: suppose the selected `+` capability has target
`x_ \.natural.+./ y_`. In a context where `m` and `n` are naturals:

```text
EFFECTIVE_KEY(m :+: n, context, registry)
    = "m \.natural.+./ n"
```

If target resolution recursively comes back to `m :+: n`, the recursion guard
returns the raw key `m :+: n` rather than recursing forever.

## Determining an expression's type

There are three related operations:

1. `KNOWN_FACTS` answers “what facts should be reported for this expression?”
   It is used for recorded type information and hover output.
2. `HAS_TYPE_SIGNATURE` answers “does this subject have this nominal type
   signature?” It is used to select type-owned capabilities.
3. `PROVE_FACT` answers “does the current state imply this arbitrary fact?” It
   is used to check `when:` requirements and selected targets.

Keeping them separate matters. A value can satisfy a supertype without the
checker replacing its directly declared type. Specification reduction and
numeric-literal fallback are also part of general proof, but not part of every
nominal owner-type lookup.

### Reporting known facts

```text
function KNOWN_FACTS(
    expression: Expression,
    context: TypeContext,
    registry: SignatureRegistry,
) -> List<String>:
    subject: Key := EFFECTIVE_KEY(expression, context, registry)
    resolving: Set<(Key, Key)> := {}
    facts: List<TypeFact> := RESULT_FACTS(
        expression, subject, context, registry, resolving,
    )

    if facts is empty:
        normalized: Key := NORMALIZE_KEY(subject, context)
        facts := [
            fact
            for fact in context.facts
            if SUBJECT(fact) = subject or SUBJECT(fact) = normalized
        ]

    if facts is empty:
        asserted: Option<TypeFact> := FACT_ASSERTED_BY(expression, context)
        if asserted is Some(fact):
            return ["asserts " + FORMAT_FACT(fact)]

        if IS_STATEMENT_SHAPED(expression)
           or RESOLVES_TO_STATEMENT_COMMAND(subject, registry):
            return ["is \\statement"]

    rendered: List<String> := MAP(FORMAT_FACT_AS_PREDICATE, facts)
    return SORT(DEDUPLICATE(rendered))
```

`RESULT_FACTS` is defined by expression shape:

```text
function RESULT_FACTS(
    expression: Expression,
    result_subject: Key,
    context: TypeContext,
    registry: SignatureRegistry,
    resolving: Set<(Key, Key)>,
) -> List<TypeFact>:
    if expression is Grouped(inner) or Labeled(inner):
        return RESULT_FACTS(inner, result_subject, context, registry, resolving)

    guard: (Key, Key) := (RENDER(expression), result_subject)
    if guard in resolving:
        return []
    add guard to resolving

    facts := match expression:
        FunctionCall(name, arguments) =>
            INSTANTIATE_FUNCTION_OUTPUTS(name, arguments, result_subject,
                                         context, registry)

        MemberCall(owner, name, arguments) =>
            MEMBER_RESULT_FACTS(owner, name, arguments, result_subject,
                                context, registry, resolving)

        MemberAccess(owner, name) =>
            MEMBER_RESULT_FACTS(owner, name, [], result_subject,
                                context, registry, resolving)

        Binary(left, operator, right) =>
            BINARY_RESULT_FACTS(left, operator, right, result_subject,
                                context, registry, resolving)

        Command(command) =>
            INSTANTIATE_DECLARED_OUTPUTS(
                outputs = DEFINED_OUTPUTS(EFFECTIVE_KEY(expression, context,
                                                        registry)),
                arguments = ARGUMENTS(command),
                result_subject,
                context,
                registry,
            )

        InfixCommand(left, command, right) =>
            INSTANTIATE_DECLARED_OUTPUTS(
                outputs = DEFINED_OUTPUTS(EFFECTIVE_KEY(expression, context,
                                                        registry)),
                arguments = [left] + ARGUMENTS(command) + [right],
                result_subject,
                context,
                registry,
            )

        otherwise => []

    remove guard from resolving
    return facts
```

Function outputs and declared command outputs are instantiated as follows:

```text
function INSTANTIATE_FUNCTION_OUTPUTS(
    name: Key,
    arguments: List<Expression>,
    result_subject: Key,
    context: TypeContext,
    registry: SignatureRegistry,
) -> List<TypeFact>:
    results: List<TypeFact> := []

    for fact in INFERRED_FUNCTION_TYPES(name, context, registry):
        let FunctionType(_, inputs, output, variadic_tuple) = fact
        argument_keys := MAP(RENDER, arguments)
        matched := MATCH_FUNCTION_ARGUMENTS(
            LENGTH(inputs), variadic_tuple, argument_keys,
        )
        if matched is None:
            continue

        # Input requirements are validated by the expression-checking path.
        # Result discovery needs only a compatible argument shape.
        APPEND(results, NORMALIZE_FACT(
            INSTANTIATE_FUNCTION_SPEC(output, result_subject), context,
        ))

    return results

function INSTANTIATE_DECLARED_OUTPUTS(
    outputs: List<TypeFact>,
    arguments: List<Expression>,
    result_subject: Key,
    context: TypeContext,
    registry: SignatureRegistry,
) -> List<TypeFact>:
    child: TypeContext := CLONE(context)
    for output in outputs:
        ADD_FACT(child, output)

    inferred: List<TypeFact> := []
    argument_keys := [
        EFFECTIVE_KEY(argument, child, registry)
        for argument in arguments
    ]

    for direct in outputs:
        for function_type in INFERRED_FUNCTION_TYPES(
            SUBJECT(direct), child, registry,
        ):
            let FunctionType(_, inputs, output, variadic_tuple) = function_type
            matched := MATCH_FUNCTION_ARGUMENTS(
                LENGTH(inputs), variadic_tuple, argument_keys,
            )
            if matched is None:
                continue

            instantiated_inputs := [
                INSTANTIATE_FUNCTION_SPEC(spec, argument_key)
                for (spec, argument_key) in ZIP(inputs, matched.value)
            ]
            if every PROVE_FACT(input, child, registry)
               for input in instantiated_inputs:
                APPEND(inferred, NORMALIZE_FACT(
                    INSTANTIATE_FUNCTION_SPEC(output, result_subject), child,
                ))

    if inferred is not empty:
        return SORT_AND_DEDUPLICATE(inferred)

    # No callable output was inferred. Preserve direct declaration outputs,
    # but make each one a fact about this particular result expression.
    return [
        REPLACE_SUBJECT(output, result_subject, context)
        for output in outputs
    ]
```

`INFERRED_FUNCTION_TYPES` starts with function facts in the context and follows
the output facts and extension rules of the type named by each fact. It uses a
fact-level cycle guard. `MATCH_FUNCTION_ARGUMENTS` accepts either the exact
input count or the tuple shape required by a variadic-tuple input.

For a function type such as:

```text
FunctionType(
    subject = "f",
    inputs = [IsSpec("\real", "\real")],
    output = IsSpec("\integer", "\integer"),
    variadic_tuple_input = false,
)
```

`RESULT_FACTS(f(r), "f(r)", ...)` instantiates the output as
`Is("f(r)", "\integer", "\integer")`. For a plain name such as `r`,
`RESULT_FACTS` returns no facts, so `KNOWN_FACTS` reports the facts attached to
`r` in `context.facts`. For the expression `r is? \real`, no value fact is
available, so `KNOWN_FACTS` reports that the expression asserts the corresponding
fact.

### Testing for a nominal type signature

```text
function HAS_TYPE_SIGNATURE(
    subject: Key,
    required_signature: TypeSignature,
    context: TypeContext,
    registry: SignatureRegistry,
    allow_views: Boolean = false,
) -> Boolean:
    subject := NORMALIZE_KEY(subject, context)
    seen: Set<TypeFact> := {}

    candidates: List<TypeFact> := context.facts
    candidates += DEFINED_OUTPUT_FACTS_FOR_KEY(subject, context, registry)

    # A view declared directly by a Defines:/Realizes: value is an initial
    # candidate. `allow_views` controls further recursive view traversal.
    candidates += DEFINED_VALUE_VIEW_FACTS_FOR_KEY(subject, context, registry)

    return any FACT_HAS_TYPE_SIGNATURE(
        fact, subject, required_signature, context, registry, seen, allow_views,
    ) for fact in candidates

function FACT_HAS_TYPE_SIGNATURE(
    candidate: TypeFact,
    subject: Key,
    required_signature: TypeSignature,
    context: TypeContext,
    registry: SignatureRegistry,
    seen: Set<TypeFact>,
    allow_views: Boolean,
) -> Boolean:
    fact: TypeFact := NORMALIZE_FACT(candidate, context)
    if fact in seen:
        return false
    add fact to seen

    if fact is Is(fact_subject, _, fact_signature)
       and fact_subject = subject
       and fact_signature = required_signature:
        return true

    if fact is RefinedIs(fact_subject, _, refined_signature, _, _)
       and fact_subject = subject
       and refined_signature = required_signature:
        return true

    if fact is Is(fact_subject, _, fact_signature)
       and fact_subject = subject
       and SIGNATURES_ARE_EQUIVALENT(fact_signature, required_signature,
                                     registry):
        return true

    derived: List<TypeFact> := []
    derived += INSTANTIATED_TYPE_REQUIREMENTS(fact, context, registry)
    derived += DIRECT_EXTENSION_FACTS(fact, context, registry)
    derived += REFINED_BASE_PARTS_AND_EXTENSIONS(fact, context, registry,
                                                allow_views)
    if allow_views:
        derived += VIEW_FACTS_FROM(fact, context, registry)

    return any FACT_HAS_TYPE_SIGNATURE(
        next, subject, required_signature, context, registry, seen, allow_views,
    ) for next in derived
```

`INSTANTIATED_TYPE_REQUIREMENTS` substitutes a concrete type's actual arguments
into the requirements recorded on its declaration. Requirements involving a
`using:` parameter are excluded because that parameter is unavailable from the
type fact alone. Hidden parameters receive stable internal placeholder keys.

For a defined command with a direct view, the initial candidate rule is
intentional. If `\naturals` is a `Defines:` value with `Enables: view: as: X :=
N is \set`, then `HAS_TYPE_SIGNATURE("\naturals", "\set", ..., false)` can
succeed from that advertised direct view. By contrast, recursively following a
view owned by an ordinary type fact requires `allow_views = true`.

For example, suppose `\group` extends `\magma`, and the context contains:

```text
Is("G", "\group", "\group")
```

Then `HAS_TYPE_SIGNATURE("G", "\magma", ...)` follows the registered extension
fact and returns `true`. `KNOWN_FACTS(G, ...)` may still report `G is \group`;
the successful query does not replace that direct fact with `G is \magma`.

As a refinement example, if `x` has a `RefinedIs` fact for
`\(positive)::real`, then a lookup for the base signature `\real` succeeds
after `REFINED_BASE_PARTS_AND_EXTENSIONS` produces `x is \real`.

The cycle guard is essential. If `\alpha` extends `\beta` and `\beta` extends
`\alpha`, a failed lookup for `\gamma` visits each normalized fact at most once
and returns `false`.

### Proving an arbitrary requirement

General proof accepts every `TypeFact` form, not only nominal types.

```text
function PROVE_FACT(
    required: TypeFact,
    context: TypeContext,
    registry: SignatureRegistry,
    allow_views: Boolean = true,
) -> Boolean:
    required := NORMALIZE_FACT(required, context)

    if BUILTIN_FACT_HOLDS(required, registry):
        return true
    if required equals a normalized fact in context.facts:
        return true
    if required is a composite RefinedIs and
       its base plus every individual refinement can be proved:
        return true

    candidates: List<TypeFact> :=
        DEFINED_OUTPUT_FACTS_FOR_KEY(SUBJECT(required), context, registry)
        + context.facts

    if allow_views:
        candidates += DEFINED_VALUE_VIEW_FACTS_FOR_KEY(
            SUBJECT(required), context, registry,
        )

    seen: Set<TypeFact> := {}
    if any FACT_IMPLIES(candidate, required, context, registry, seen,
                        allow_views) for candidate in candidates:
        return true

    if allow_views and a simple component view rewrites required to a fact
       that can be proved:
        return true

    if SUBJECT(required) is not a declared name and
       NUMERIC_LITERAL_FACT(SUBJECT(required), registry) is Some(fallback) and
       FACT_IMPLIES(fallback, required, context, registry, seen, allow_views):
        return true

    if required is a reversible Spec or reversible literal MemberOf fact:
        return every fact on the reverse side can be proved

    return false

function FACT_IMPLIES(
    candidate: TypeFact,
    required: TypeFact,
    context: TypeContext,
    registry: SignatureRegistry,
    seen: Set<TypeFact>,
    allow_views: Boolean,
) -> Boolean:
    fact: TypeFact := NORMALIZE_FACT(candidate, context)
    if fact = required:
        return true
    if fact in seen:
        return false
    add fact to seen

    if FUNCTION_OUTPUT_IMPLIES(fact, required, context, registry, seen,
                               allow_views):
        return true
    if CAST_RULE_IMPLIES(fact, required, context, registry, seen, allow_views):
        return true
    if allow_views and VIEW_RULE_IMPLIES(fact, required, context, registry,
                                         seen):
        return true
    if EQUIVALENCE_IMPLIES(fact, required, context, registry):
        return true

    derived: List<TypeFact> := DIRECT_EXTENSION_FACTS(fact, context, registry)
    derived += REFINED_BASE_PARTS_AND_EXTENSIONS(fact, context, registry,
                                                allow_views)
    if fact is Spec or MemberOf:
        derived += REDUCE_SPEC_OR_MEMBERSHIP(fact, context, registry)

    return any FACT_IMPLIES(next, required, context, registry, seen,
                            allow_views) for next in derived
```

Example: given `x "in" M`, suppose `\magma` enables a specification rule that
reduces membership in `M` to `x is \magma.element:of{M}`. `PROVE_FACT` can use
that reduction to establish the latter fact. `HAS_TYPE_SIGNATURE` does not
perform this reduction on its own; a caller that needs owner lookup from a spec
must first materialize the reduced facts.

Numeric literals use another general-proof fallback. If the collection-wide
configuration contains:

```text
Specify:
. positiveInt:
  is: \natural
```

then an otherwise undeclared `3` can satisfy `3 is \natural`. If the lexical
context explicitly declares a symbol named `3`, its local facts take precedence
and the fallback is not used.

Disambiguation calls `PROVE_FACT(..., allow_views = false)`. Consequently a view
may help validate the requirements of an already selected target but cannot make
a `when:` branch win.

### Materializing specification and membership reductions

Some callers need the consequences of every `Spec` and `MemberOf` fact to be
available to nominal type lookup. They build a derived context with a work-list
algorithm:

```text
function MATERIALIZE_SPEC_AND_MEMBERSHIP_FACTS(
    context: TypeContext,
    registry: SignatureRegistry,
) -> TypeContext:
    child: TypeContext := CLONE(context)
    known: Set<TypeFact> := {
        NORMALIZE_FACT(fact, child) for fact in child.facts
    }
    index: Integer := 0

    while index < LENGTH(child.facts):
        fact: TypeFact := child.facts[index]
        index := index + 1

        if fact is not Spec and fact is not MemberOf:
            continue

        reduction_seen: Set<TypeFact> := {}
        reduced: List<TypeFact> := REDUCE_SPEC_OR_MEMBERSHIP_WITH_GUARD(
            fact, child, registry, reduction_seen,
        )
        for candidate in reduced:
            normalized := NORMALIZE_FACT(candidate, child)
            if normalized not in known:
                add normalized to known
                ADD_FACT(child, normalized)

    return child
```

The loop also processes newly appended `Spec` and `MemberOf` facts, so a chain
such as `x "in" A -> x member_of B -> x is \element:of{B}` reaches its final
type fact. Both `known` and `reduction_seen` are needed: the former avoids adding
duplicate work, while the latter stops cycles within one recursive reduction.
The original context is unchanged.

## Resolving a binary operator

The four spellings select different resolution strategies:

| Source form | Ownership mode | Resolution source |
| --- | --- | --- |
| `a + b` | `Plain` | bound `+`, then `Disambiguates`, then common owner |
| `a :+ b` | `Left` | a provided `+` capability owned by the type of `a` |
| `a +: b` | `Right` | a provided `+` capability owned by the type of `b` |
| `a :+: b` | `Both` | a provided `+` capability whose owner type both satisfy |

The colon points at the owning operand: the colon in `a :+ b` is beside `a`,
and the colon in `a +: b` is beside `b`.

The same algorithm applies to the other symbolic operators and to
colon-decorated named operators. Plain named operators are different: `a |plus|
b` is always application sugar for `plus(a, b)`.

The resolution data used below is:

```text
enum Ownership:
    Left
    Right
    Both

enum DisambiguationKey:
    BinaryOperator(name: OperatorName)
    Function(name: String, arity: Integer)
    PrefixOperator(name: OperatorName)
    PostfixOperator(name: OperatorName)

record PreparedOperands:
    expressions: List<Expression>
    actual_keys: List<Key>
    context: TypeContext

record Resolution:
    target: Expression
    checking_context: TypeContext
    source: BoundCall | DisambiguationBranch | ProvidedSymbolRule | Builtin

enum ResolutionError:
    UnrecognizedSymbol(Key)
    NoDisambiguation(DisambiguationKey)
    NoMatchingBranch(DisambiguationKey, List<Key>)
    NoProvidedCapability(DisambiguationKey, Ownership, List<Key>)
    RecursiveResolution(DisambiguationKey)
    InvalidTarget(List<Diagnostic>)
```

Operator and function keys have deliberately equivalent forms:

```text
function DISAMBIGUATION_KEYS_MATCH(
    left: DisambiguationKey,
    right: DisambiguationKey,
) -> Boolean:
    if left = right:
        return true

    equivalent: List<DisambiguationKey> := match left:
        BinaryOperator(operator) => [
            Function(FUNCTION_NAME_FOR_OPERATOR(operator), 2),
        ]
        Function(name, 2) => [
            BinaryOperator(UNSTROP_NAME(name)),
        ]
        PrefixOperator(operator) => [
            Function(FUNCTION_NAME_FOR_OPERATOR(operator), 1),
        ]
        PostfixOperator(operator) => [
            Function(FUNCTION_NAME_FOR_OPERATOR(operator), 1),
        ]
        Function(name, 1) => [
            PrefixOperator(UNSTROP_NAME(name)),
            PostfixOperator(UNSTROP_NAME(name)),
        ]
        otherwise => []

    return right in equivalent
```

`FUNCTION_NAME_FOR_OPERATOR("+")` returns the stropped name `` `+` ``, while an
operator name that is already a plain identifier needs no stropping. Thus a
binary `+` rule and a two-argument `` `+`(...) `` form share a lookup identity,
but neither matches a unary `+` rule.

### Preparing operands

```text
function PREPARE_OPERANDS(
    expressions: List<Expression>,
    context: TypeContext,
    registry: SignatureRegistry,
) -> PreparedOperands:
    prepared: TypeContext := CLONE(context)

    for expression in expressions:
        prepared += EXPLICIT_CAST_FACTS_WITHIN(expression)

    # Process left-to-right so the result facts of an earlier composite
    # expression are visible while resolving a later expression.
    for expression in expressions:
        subject: Key := EFFECTIVE_KEY(expression, prepared, registry)
        for fact in RESULT_FACTS(expression, subject, prepared, registry, {}):
            ADD_FACT(prepared, fact)

    actuals: List<Key> := [
        EFFECTIVE_KEY(expression, prepared, registry)
        for expression in expressions
    ]

    return PreparedOperands(expressions, actuals, prepared)
```

Example: if `makeNatural()` has result fact `makeNatural() is \natural`, then
preparing `[makeNatural(), n]` adds that fact before capability matching. A
common-owned natural capability can therefore match the composite left operand;
it is not limited to plain names already present in the original context.

### Plain `a + b`

```text
function RESOLVE_PLAIN_OPERATOR(
    left: Expression,
    symbol: OperatorName,
    right: Expression,
    context: TypeContext,
    registry: SignatureRegistry,
) -> Result<Resolution, ResolutionError>:
    if context contains a bound symbol named symbol:
        call: Expression := DESUGAR_AS_CALL(symbol, [left, right])
        return RESOLVE_AND_CHECK_CALL(call, context, registry)

    prepared := PREPARE_OPERANDS([left, right], context, registry)
    key := BinaryOperator(symbol)
    entry: Option<DisambiguationRule> := FIRST_MATCHING_DISAMBIGUATION(
        key, registry.disambiguations,
    )

    if entry is Some(rule):
        if LENGTH(rule.parameters) != 2:
            return Err(NoMatchingBranch(key, prepared.actual_keys))
        if key matches an entry in prepared.context.active_disambiguations:
            return Err(RecursiveResolution(key))

        parameter_bindings: Map<Key, Key> := ZIP_MAP(
            rule.parameters,
            MAP(key => NORMALIZE_KEY(key, prepared.context),
                prepared.actual_keys),
        )

        for branch in rule.branches in source order:
            branch_context := CLONE(prepared.context)
            ADD_INSTANTIATED_BRANCH_SUBSTITUTIONS(
                branch_context, branch.substitutions, parameter_bindings,
            )
            requirements := [
                SUBSTITUTE_FACT(requirement, parameter_bindings)
                for requirement in branch.requirements
            ]

            if every PROVE_FACT(requirement, branch_context, registry,
                                allow_views = false)
               for requirement in requirements:
                return CHECK_DISAMBIGUATION_TARGET(
                    rule, branch.target, parameter_bindings,
                    branch.substitutions, prepared.context, registry,
                )

        if rule.else_expression is Some(fallback):
            return CHECK_DISAMBIGUATION_TARGET(
                rule, fallback, parameter_bindings, [], prepared.context,
                registry,
            )

        return Err(NoMatchingBranch(key, prepared.actual_keys))

    # This fallback is available only when no Disambiguates entry owns `key`.
    reduced_context := MATERIALIZE_SPEC_AND_MEMBERSHIP_FACTS(
        prepared.context, registry,
    )
    reduced_actuals := [
        EFFECTIVE_KEY(left, reduced_context, registry),
        EFFECTIVE_KEY(right, reduced_context, registry),
    ]
    return RESOLVE_PROVIDED_OPERATOR(
        key, Both, reduced_actuals, reduced_context, registry,
    )
```

The helper used for a selected branch or `else:` target installs the same
bindings used during branch matching, and marks the disambiguation active while
checking its target:

```text
function CHECK_DISAMBIGUATION_TARGET(
    rule: DisambiguationRule,
    target: Expression,
    parameter_bindings: Map<Key, Key>,
    branch_substitutions: List<(Key, Key)>,
    context: TypeContext,
    registry: SignatureRegistry,
) -> Result<Resolution, ResolutionError>:
    child: Option<TypeContext> := ACTIVATE_DISAMBIGUATION(context, rule.key)
    if child is None:
        return Err(RecursiveResolution(rule.key))

    for parameter in rule.parameters:
        DECLARE_NAME(child.value, parameter)
        ADD_SUBSTITUTION(
            child.value, parameter, parameter_bindings[parameter],
        )

    for (left, right) in branch_substitutions:
        ADD_SUBSTITUTION(
            child.value,
            SUBSTITUTE_KEY(left, parameter_bindings),
            SUBSTITUTE_KEY(right, parameter_bindings),
        )

    diagnostics := CHECK_EXPRESSION(target, child.value, registry)
    if diagnostics is not empty:
        return Err(InvalidTarget(diagnostics))

    return Ok(Resolution(
        target = target,
        checking_context = child.value,
        source = the selected branch or else arm,
    ))
```

Activating the key prevents a target such as `else: x_ + y_` from recursively
selecting the same disambiguation forever. A target can still use a different
operator or function disambiguation.

For a concrete branch example:

```text
[x_ + y_]
Disambiguates:
when:
. x_ is \real
. y_ is \complex
to: x_ \.complex.+./ y_
when:
. x_ is \real
. y_ is \integer
to: x_ \.real.+./ y_
```

With `r is \real` and `n is \integer`, the parameter bindings are `{x_ -> r,
y_ -> n}`. The first branch fails its second requirement. The second branch
succeeds and selects `r \.real.+./ n`. If `n` also happened to satisfy
`\complex`, the first branch would win because branches are ordered; there is no
specificity ranking.

A `Disambiguates:` entry owns resolution completely. If it exists but no branch
matches and it has no `else:`, the checker does not fall through to a provided
capability.

Plain `=` and `!=` are special. The checker first tries a common-owner provided
capability. If none exists, it accepts the expression as a built-in statement
for arbitrary operand types. For example, `x = y` is statement-shaped even if
the types of `x` and `y` provide no equality capability.

### Left-owned `a :+ b`

```text
function RESOLVE_LEFT_OWNED(
    left: Expression,
    symbol: OperatorName,
    right: Expression,
    context: TypeContext,
    registry: SignatureRegistry,
) -> Result<Resolution, ResolutionError>:
    prepared := PREPARE_OPERANDS([left, right], context, registry)
    return RESOLVE_PROVIDED_OPERATOR(
        BinaryOperator(symbol), Left, prepared.actual_keys,
        prepared.context, registry,
    )
```

Example: suppose `\set` enables `x_ - y_ :=> x_ \.set.minus./ y_`, `A is
\set`, and `n is \natural`. `A :- n` may select the set-owned capability because
only `A`, the left owner, must satisfy `\set`. Checking the instantiated target
can still reject the expression if `\.set.minus./` requires its right argument
to be a set.

### Right-owned `a +: b`

```text
function RESOLVE_RIGHT_OWNED(
    left: Expression,
    symbol: OperatorName,
    right: Expression,
    context: TypeContext,
    registry: SignatureRegistry,
) -> Result<Resolution, ResolutionError>:
    prepared := PREPARE_OPERANDS([left, right], context, registry)
    return RESOLVE_PROVIDED_OPERATOR(
        BinaryOperator(symbol), Right, prepared.actual_keys,
        prepared.context, registry,
    )
```

With the same set capability, `n -: A` may select it because `A`, the right
owner, satisfies `\set`. `n :- A` cannot select that capability from `A` because
the left-owned spelling points at `n`.

### Common-owned `a :+: b`

```text
function RESOLVE_COMMON_OWNED(
    left: Expression,
    symbol: OperatorName,
    right: Expression,
    context: TypeContext,
    registry: SignatureRegistry,
) -> Result<Resolution, ResolutionError>:
    prepared := PREPARE_OPERANDS([left, right], context, registry)
    return RESOLVE_PROVIDED_OPERATOR(
        BinaryOperator(symbol), Both, prepared.actual_keys,
        prepared.context, registry,
    )
```

Both operands must satisfy one rule's owner signature. Their written type keys
need not be identical. For example, if `\integer` extends `\real`, then `i is
\integer` and `r is \real` can both satisfy a `\real`-owned capability. The
checker does not construct or rank a least common ancestor; it scans registered
provided-symbol rules and selects the first match.

Unlike the plain-operator fallback, the explicit `:+`, `+:`, and `:+:` checking
paths do not first materialize `Spec` or `MemberOf` reductions. Thus a context
containing only `x "in" M` may let plain `x + x` find an element-owned
capability after reduction, while `x :+: x` requires the needed owner type fact
to already be available to nominal type lookup.

### Selecting a provided capability

```text
function RESOLVE_PROVIDED_OPERATOR(
    key: DisambiguationKey,
    ownership: Ownership,
    actuals: List<Key>,
    context: TypeContext,
    registry: SignatureRegistry,
) -> Result<Resolution, ResolutionError>:
    for rule in registry.provided_symbols in registration order:
        if not DISAMBIGUATION_KEYS_MATCH(key, rule.key):
            continue
        if LENGTH(rule.parameters) != LENGTH(actuals):
            continue

        owner: Key := match ownership:
            Left  => actuals[0]
            Right => actuals[LENGTH(actuals) - 1]
            Both  => actuals[0]  # binding anchor; every actual is checked below

        owner_matches: Boolean := match ownership:
            Left => HAS_TYPE_SIGNATURE(
                actuals[0], rule.owner_signature, context, registry,
            )
            Right => HAS_TYPE_SIGNATURE(
                actuals[LENGTH(actuals) - 1], rule.owner_signature,
                context, registry,
            )
            Both => every HAS_TYPE_SIGNATURE(
                actual, rule.owner_signature, context, registry,
            ) for actual in actuals

        if not owner_matches:
            continue
        if rule.source_subject is Some(_)
           and COLLECTION_LITERAL(owner, context) is None:
            continue

        child: TypeContext := CLONE(context)
        for formal in rule.parameters:
            DECLARE_NAME(child, formal)
        for (formal, actual) in ZIP(rule.parameters, actuals):
            ADD_SUBSTITUTION(child, formal, NORMALIZE_KEY(actual, context))

        owner_type_actuals: Option<List<Key>> := TYPE_ACTUALS_FOR_SIGNATURE(
            owner, rule.owner_signature, context, registry,
        )
        if owner_type_actuals is Some(type_actuals):
            owner_info := registry.type_infos[rule.owner_signature]
            for (formal, actual) in ZIP(owner_info.parameters, type_actuals):
                DECLARE_NAME(child, formal)
                ADD_SUBSTITUTION(child, formal,
                                 NORMALIZE_KEY(actual, context))

        DECLARE_NAME(child, rule.owner_subject)
        ADD_SUBSTITUTION(child, rule.owner_subject,
                         NORMALIZE_KEY(owner, context))

        if rule.source_subject is Some(source):
            DECLARE_NAME(child, source)
            ADD_SUBSTITUTION(child, source, NORMALIZE_KEY(owner, context))

        BIND_OWNER_DESTRUCTURED_COMPONENTS(rule, child, registry)

        diagnostics := CHECK_EXPRESSION(rule.target, child, registry)
        if diagnostics is not empty:
            return Err(InvalidTarget(diagnostics))

        return Ok(Resolution(
            target = rule.target,
            checking_context = child,
            source = rule,
        ))

    return Err(NoProvidedCapability(key, ownership, actuals))
```

Consider this simplified declaration:

```text
[\element:of{M ::= (X, *)}]
Declares: x "in" X
when: M is \magma
Enables:
. capability: x_ * y_ :=> x_ |M.*| y_
```

For `a :+: b`, where both operands have type `\element:of{G}` and `G ::= (S,
op)` is a magma, the selected rule creates these effective bindings:

```text
x_              -> a
y_              -> b
x               -> a    # the type declaration's described owner subject
M               -> G    # actual argument of \element:of{}
components of G = [S, op]
```

The type metadata says that `*` is the second component of the destructured
`M`. After `M -> G`, member lookup therefore resolves `G.*` to `op`. The target
`a |G.*| b` is consequently checked as the ordinary application `op(a, b)`.
The formal-to-actual bindings are reapplied directionally when an effective key
or output fact escapes the child context. This matters because the context's
equivalence-class normalization might otherwise choose a formal name such as
`x_` as its representative instead of the call-site name `a`.

The literal-source test handles a capability declared under `from:`. Such a rule
matches only when the selected owner is a collection literal; it is not a
general capability of all values having the same nominal type.

### Determining the resolved operator's result facts

```text
function PROVIDED_OPERATOR_RESULT_FACTS(
    rule: ProvidedSymbolRule,
    actuals: List<Key>,
    owner: Option<Key>,
    result_subject: Key,
    context: TypeContext,
    registry: SignatureRegistry,
    resolving: Set<(Key, Key)>,
) -> List<TypeFact>:
    child: TypeContext := BUILD_PROVIDED_TARGET_CONTEXT(
        rule, actuals, owner, context, registry,
    )

    directional: Map<Key, Key> := FORMAL_TO_ACTUAL_BINDINGS(
        rule, actuals, owner, context,
    )

    target_facts: List<TypeFact> := RESULT_FACTS(
        rule.target, result_subject, child, registry, resolving,
    )
    return [
        SUBSTITUTE_FACT(fact, directional)
        for fact in target_facts
    ]
```

Operator result facts are not guessed from the operands. They come from the
selected target, after the same operand, owner-type-parameter, source, and
destructuring bindings used to check that target.

For example, if the natural-owned `+` capability targets
`x_ \.natural.+./ y_`, and that command declares a result `z is \natural`, then
the result facts for `m :+: n` contain:

```text
Is(
    subject = "m \.natural.+./ n",
    type_key = "\natural",
    signature = "\natural",
)
```

If a plain operator selects a `Disambiguates:` target, its effective key is the
instantiated target key. Defined output lookup and any provided operator inside
that target then use the selected target's declared result facts. For example,
an `else: x_ :-: y_` target obtains its result from the set-owned `:-:`
capability, not from the undecorated `-` expression.

## End-to-end examples

### Plain operator resolved by `Disambiguates`

Assume `r is \real`, `n is \integer`, and the two-branch `+` disambiguation from
the earlier example.

```text
PREPARE_OPERANDS([r, n])
    actual_keys = ["r", "n"]

bindings
    x_ -> r
    y_ -> n

branch 1 requirements
    r is \real       => true
    n is \complex    => false

branch 2 requirements
    r is \real       => true
    n is \integer    => true

selected target
    r \.real.+./ n
```

### Plain operator resolved from a reduced specification

Assume the only local fact about `x` is `x "in" M`. The registry says that
membership in a magma reduces to `x is \element:of{M}`, and `\element:of{}`
provides `+`.

```text
original context fact
    Spec("x", "in", "M")

MATERIALIZE_SPEC_AND_MEMBERSHIP_FACTS
    adds Is("x", "\element:of{M}", "\element:of{}")

HAS_TYPE_SIGNATURE("x", "\element:of{}")
    => true

plain x + x
    => selects the first matching common-owner capability
```

### A view does not select an overload

Assume `N` is directly typed as `\naturals.structure` and has a view exposing a
component as `\set`. A disambiguation branch requires `x_ is \set`.

```text
PROVE_FACT(N is \set, allow_views = true)  => true
PROVE_FACT(N is \set, allow_views = false) => false
```

Therefore `N` can satisfy a normal command requirement for `\set`, but that
view alone cannot make the `\set` branch of an overloaded operator win.

## Failure summary

Resolution fails in the following cases:

| Cause | Concrete example |
| --- | --- |
| An operand or bound operator is out of scope. | `x + y` is checked where `y` was never declared. |
| An explicitly owned form has no capability on the selected owner. | `n :- A` points at `n`, but only `\set` provides `-` and `n` is a `\natural`. |
| A common-owned form finds no one owner signature satisfied by every operand. | `A :-: n` when `A` is only a `\set`, `n` is only a `\natural`, and neither type owns both. |
| A `Disambiguates:` entry has no matching branch and no `else:`. | Only real-plus-integer is declared, but the call is `real + complex`. |
| A selected target's own requirements fail. | A left-owned set capability is selected by `A :- n`, but its target requires both arguments to be sets. |
| A literal-source capability receives a non-literal owner. | A `from: Y ::= {...}` capability is attempted with an opaque set variable. |
| Resolution cycles without reaching an independently known target or fact. | A `+` disambiguation targets the same unresolved `+` with the same arguments. |

Diagnostics distinguish “no matching `Disambiguates` entry,” “could not
disambiguate these arguments,” “could not resolve from the selected operand
type,” and errors emitted while checking an already selected target. Keeping
those stages separate makes it possible to tell failure to select an overload
from failure inside the overload that was selected.
