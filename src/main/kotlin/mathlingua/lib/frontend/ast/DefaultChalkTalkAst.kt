package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.MetaData

internal val DEFAULT_METADATA = MetaData(row = -1, column = -1, isInline = true)

internal val DEFAULT_TEXT_BLOCK = TextBlock(text = "", metadata = DEFAULT_METADATA)

internal val DEFAULT_ID = Id(text = "", metadata = DEFAULT_METADATA)

internal val DEFAULT_AND_SECTION = AndSection(clauses = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_AND_GROUP =
    AndGroup(andSection = DEFAULT_AND_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_CLAUSE: Clause = DEFAULT_AND_GROUP

internal val DEFAULT_NOT_SECTION = NotSection(clause = DEFAULT_CLAUSE, metadata = DEFAULT_METADATA)

internal val DEFAULT_NOT_GROUP =
    NotGroup(notSection = DEFAULT_NOT_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_OR_SECTION = OrSection(clauses = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_OR_GROUP = OrGroup(orSection = DEFAULT_OR_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_EXISTS_SECTION =
    ExistsSection(targets = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_WHERE_SECTION = WhereSection(specs = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_SUCH_THAT_SECTION =
    SuchThatSection(clauses = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_EXISTS_GROUP =
    ExistsGroup(
        existsSection = DEFAULT_EXISTS_SECTION,
        whereSection = null,
        suchThatSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_EXISTS_UNIQUE_SECTION =
    ExistsUniqueSection(targets = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_EXISTS_UNIQUE_GROUP =
    ExistsUniqueGroup(
        existsUniqueSection = DEFAULT_EXISTS_UNIQUE_SECTION,
        whereSection = null,
        suchThatSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_FOR_ALL_SECTION =
    ForAllSection(targets = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_THEN_SECTION = ThenSection(clauses = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_FOR_ALL_GROUP =
    ForAllGroup(
        forAllSection = DEFAULT_FOR_ALL_SECTION,
        whereSection = null,
        suchThatSection = null,
        thenSection = DEFAULT_THEN_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_IF_SECTION = IfSection(clauses = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_IF_GROUP =
    IfGroup(
        ifSection = DEFAULT_IF_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_IFF_SECTION = IffSection(clauses = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_IFF_GROUP =
    IffGroup(
        iffSection = DEFAULT_IFF_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_GENERATED_SECTION = GeneratedSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_FROM_SECTION = FromSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_GENERATED_WHEN_SECTION =
    GeneratedWhenSection(statements = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_GENERATED_GROUP =
    GeneratedGroup(
        generatedSection = DEFAULT_GENERATED_SECTION,
        fromSection = DEFAULT_FROM_SECTION,
        whenSection = DEFAULT_GENERATED_WHEN_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_PIECEWISE_SECTION = PiecewiseSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_PIECEWISE_WHEN_SECTION =
    PiecewiseWhenSection(clauses = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_PIECEWISE_THEN_SECTION =
    PiecewiseThenSection(statements = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_PIECEWISE_ELSE_SECTION =
    PiecewiseElseSection(statements = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_PIECEWISE_GROUP =
    PiecewiseGroup(
        piecewiseSection = DEFAULT_PIECEWISE_SECTION,
        whenSection = null,
        thenSection = null,
        piecewiseElseSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_MATCHING_SECTION =
    MatchingSection(statements = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_MATCHING_GROUP =
    MatchingGroup(matchingSection = DEFAULT_MATCHING_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_EQUALITY_SECTION = EqualitySection(metadata = DEFAULT_METADATA)

internal val DEFAULT_TARGET: Target = Name(text = "", metadata = DEFAULT_METADATA)

internal val DEFAULT_BETWEEN_SECTION =
    BetweenSection(first = DEFAULT_TARGET, second = DEFAULT_TARGET, metadata = DEFAULT_METADATA)

internal val DEFAULT_STATEMENT = Statement(text = "", metadata = DEFAULT_METADATA)

internal val DEFAULT_PROVIDED_SECTION =
    ProvidedSection(statement = DEFAULT_STATEMENT, metadata = DEFAULT_METADATA)

internal val DEFAULT_EQUALITY_GROUP =
    EqualityGroup(
        equalitySection = DEFAULT_EQUALITY_SECTION,
        betweenSection = DEFAULT_BETWEEN_SECTION,
        providedSection = DEFAULT_PROVIDED_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_MEMBERSHIP_SECTION = MembershipSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_THROUGH_SECTION =
    ThroughSection(through = DEFAULT_STATEMENT, metadata = DEFAULT_METADATA)

internal val DEFAULT_MEMBERSHIP_GROUP =
    MembershipGroup(
        membershipSection = DEFAULT_MEMBERSHIP_SECTION,
        throughSection = DEFAULT_THROUGH_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_VIEW_SECTION = ViewSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_TEXT = Text(text = "", metadata = DEFAULT_METADATA)

internal val DEFAULT_AS_SECTION = AsSection(asText = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_VIA_SECTION = ViaSection(via = DEFAULT_STATEMENT, metadata = DEFAULT_METADATA)

internal val DEFAULT_BY_SECTION = BySection(by = DEFAULT_STATEMENT, metadata = DEFAULT_METADATA)

internal val DEFAULT_VIEW_GROUP =
    ViewGroup(
        viewSection = DEFAULT_VIEW_SECTION,
        asSection = DEFAULT_AS_SECTION,
        viaSection = DEFAULT_VIA_SECTION,
        bySection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_SYMBOLS_SECTION =
    SymbolsSection(names = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_SYMBOLS_WHERE_SECTION =
    SymbolsWhereSection(statements = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_SYMBOLS_GROUP =
    SymbolsGroup(
        symbolsSection = DEFAULT_SYMBOLS_SECTION,
        whereSection = DEFAULT_SYMBOLS_WHERE_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_MEMBER_SYMBOLS_SECTION =
    MemberSymbolsSection(names = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_MEMBER_SYMBOLS_WHERE_SECTION =
    MemberSymbolsWhereSection(statements = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_MEMBER_SYMBOLS_GROUP =
    MemberSymbolsGroup(
        memberSymbolsSection = DEFAULT_MEMBER_SYMBOLS_SECTION,
        whereSection = DEFAULT_MEMBER_SYMBOLS_WHERE_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_NOTE_SECTION = NoteSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_NOTE_GROUP =
    NoteGroup(noteSection = DEFAULT_NOTE_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_AUTHOR_SECTION =
    AuthorSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_AUTHOR_GROUP =
    AuthorGroup(authorSection = DEFAULT_AUTHOR_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_TAG_SECTION = TagSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_TAG_GROUP =
    TagGroup(tagSection = DEFAULT_TAG_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_REFERENCE_SECTION =
    ReferenceSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_REFERENCE_GROUP =
    ReferenceGroup(referenceSection = DEFAULT_REFERENCE_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_DEFINES_SECTION =
    DefinesSection(target = DEFAULT_TARGET, metadata = DEFAULT_METADATA)

internal val DEFAULT_WITH_SECTION =
    WithSection(assignments = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_GIVEN_SECTION =
    GivenSection(targets = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_WHEN_SECTION = WhenSection(specs = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_MEANS_SECTION =
    MeansSection(statement = DEFAULT_STATEMENT, metadata = DEFAULT_METADATA)

internal val DEFAULT_SATISFYING_SECTION =
    SatisfyingSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_EXPRESSING_SECTION =
    ExpressingSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_USING_SECTION =
    UsingSection(statements = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_WRITING_SECTION =
    WritingSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_WRITTEN_SECTION =
    WrittenSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_CALLED_SECTION =
    CalledSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_PROVIDING_SECTION =
    ProvidingSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_METADATA_SECTION =
    MetadataSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_DEFINES_GROUP =
    DefinesGroup(
        id = DEFAULT_ID,
        definesSection = DEFAULT_DEFINES_SECTION,
        withSection = null,
        givenSection = null,
        whenSection = null,
        meansSection = null,
        satisfyingSection = null,
        expressingSection = null,
        usingSection = null,
        writingSection = null,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        calledSection = null,
        providingSection = null,
        metadataSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_STATES_SECTION = StatesSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_THAT_SECTION = ThatSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_STATES_GROUP =
    StatesGroup(
        id = DEFAULT_ID,
        statesSection = DEFAULT_STATES_SECTION,
        givenSection = null,
        whenSection = null,
        thatSection = DEFAULT_THAT_SECTION,
        usingSection = null,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        calledSection = null,
        metadataSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_RESOURCE_SECTION =
    ResourceSection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_TYPE_SECTION = TypeSection(type = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_TYPE_GROUP =
    TypeGroup(typeSection = DEFAULT_TYPE_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_NAME_SECTION = NameSection(text = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_NAME_GROUP =
    NameGroup(nameSection = DEFAULT_NAME_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_HOMEPAGE_SECTION =
    HomepageSection(homepage = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_HOMEPAGE_GROUP =
    HomepageGroup(homepageSection = DEFAULT_HOMEPAGE_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_URL_SECTION = UrlSection(url = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_URL_GROUP =
    UrlGroup(urlSection = DEFAULT_URL_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_OFFSET_SECTION =
    OffsetSection(offset = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_OFFSET_GROUP =
    OffsetGroup(offsetSection = DEFAULT_OFFSET_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_RESOURCE_GROUP =
    ResourceGroup(id = "", resourceSection = DEFAULT_RESOURCE_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_AXIOM_SECTION = AxiomSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_AXIOM_GROUP =
    AxiomGroup(
        id = DEFAULT_ID,
        axiomSection = DEFAULT_AXIOM_SECTION,
        givenSection = null,
        whereSection = null,
        suchThatSection = null,
        thenSection = DEFAULT_THEN_SECTION,
        iffSection = null,
        usingSection = null,
        metadataSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_CONJECTURE_SECTION = ConjectureSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_CONJECTURE_GROUP =
    ConjectureGroup(
        id = DEFAULT_ID,
        conjectureSection = DEFAULT_CONJECTURE_SECTION,
        givenSection = null,
        whereSection = null,
        suchThatSection = null,
        thenSection = DEFAULT_THEN_SECTION,
        iffSection = null,
        usingSection = null,
        metadataSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_THEOREM_SECTION = TheoremSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_PROOF_SECTION = ProofSection(proofs = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_THEOREM_GROUP =
    TheoremGroup(
        id = DEFAULT_ID,
        theoremSection = DEFAULT_THEOREM_SECTION,
        givenSection = null,
        whereSection = null,
        suchThatSection = null,
        thenSection = DEFAULT_THEN_SECTION,
        iffSection = null,
        usingSection = null,
        proofSection = null,
        metadataSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_TOPIC_SECTION = TopicSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_CONTENT_SECTION =
    ContentSection(content = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_TOPIC_GROUP =
    TopicGroup(
        id = "",
        topicSection = DEFAULT_TOPIC_SECTION,
        contentSection = DEFAULT_CONTENT_SECTION,
        metadataSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_NOTE_TOP_LEVEL_SECTION = NoteTopLevelSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_NOTE_TOP_LEVEL_GROUP =
    NoteTopLevelGroup(
        noteSection = DEFAULT_NOTE_TOP_LEVEL_SECTION,
        contentSection = DEFAULT_CONTENT_SECTION,
        metadataSection = null,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_SPECIFY_SECTION =
    SpecifySection(items = emptyList(), metadata = DEFAULT_METADATA)

internal val DEFAULT_SPECIFY_GROUP =
    SpecifyGroup(specifySection = DEFAULT_SPECIFY_SECTION, metadata = DEFAULT_METADATA)

internal val DEFAULT_ZERO_SECTION = ZeroSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_IS_SECTION = IsSection(form = DEFAULT_TEXT, metadata = DEFAULT_METADATA)

internal val DEFAULT_ZERO_GROUP =
    ZeroGroup(
        zeroSection = DEFAULT_ZERO_SECTION,
        isSection = DEFAULT_IS_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_POSITIVE_INT_SECTION = PositiveIntSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_POSITIVE_INT_GROUP =
    PositiveIntGroup(
        positiveIntSection = DEFAULT_POSITIVE_INT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_NEGATIVE_INT_SECTION = NegativeIntSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_NEGATIVE_INT_GROUP =
    NegativeIntGroup(
        negativeIntSection = DEFAULT_NEGATIVE_INT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_POSITIVE_FLOAT_SECTION = PositiveFloatSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_POSITIVE_FLOAT_GROUP =
    PositiveFloatGroup(
        positiveFloatSection = DEFAULT_POSITIVE_FLOAT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_NEGATIVE_FLOAT_SECTION = NegativeFloatSection(metadata = DEFAULT_METADATA)

internal val DEFAULT_NEGATIVE_FLOAT_GROUP =
    NegativeFloatGroup(
        negativeFloatSection = DEFAULT_NEGATIVE_FLOAT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        metadata = DEFAULT_METADATA)

internal val DEFAULT_DOCUMENT = Document(items = emptyList())
