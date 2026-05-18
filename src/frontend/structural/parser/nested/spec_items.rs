use super::*;

/// Parses a positive-integer specification item.
///
/// The structural shape is identified by a `positive:` section together with an
/// `int:` section; all prose fields are open and may be empty.
pub(in crate::frontend::structural::parser) fn parse_positive_int(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PositiveIntGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "positive_int",
        &group.sections,
        tracker,
        &["positive", "int", "is"],
    )?;
    Some(PositiveIntGroup {
        heading,
        positive: PositiveSection {
            arguments: parse_optional_open_texts(sections.get("positive").copied(), tracker),
        },
        int: IntSection {
            arguments: parse_optional_open_texts(sections.get("int").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a negative-integer specification item.
pub(in crate::frontend::structural::parser) fn parse_negative_int(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<NegativeIntGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "negative_int",
        &group.sections,
        tracker,
        &["negative", "int", "is"],
    )?;
    Some(NegativeIntGroup {
        heading,
        negative: NegativeSection {
            arguments: parse_optional_open_texts(sections.get("negative").copied(), tracker),
        },
        int: IntSection {
            arguments: parse_optional_open_texts(sections.get("int").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a zero specification item.
pub(in crate::frontend::structural::parser) fn parse_zero(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ZeroGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("zero", &group.sections, tracker, &["zero", "is"])?;
    Some(ZeroGroup {
        heading,
        zero: ZeroSection {
            arguments: parse_optional_open_texts(sections.get("zero").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a positive-decimal specification item.
pub(in crate::frontend::structural::parser) fn parse_positive_decimal(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PositiveDecimalGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "positive_decimal",
        &group.sections,
        tracker,
        &["positive", "decimal", "is"],
    )?;
    Some(PositiveDecimalGroup {
        heading,
        positive: PositiveSection {
            arguments: parse_optional_open_texts(sections.get("positive").copied(), tracker),
        },
        decimal: DecimalSection {
            arguments: parse_optional_open_texts(sections.get("decimal").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a negative-decimal specification item.
pub(in crate::frontend::structural::parser) fn parse_negative_decimal(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<NegativeDecimalGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "negative_decimal",
        &group.sections,
        tracker,
        &["negative", "decimal", "is"],
    )?;
    Some(NegativeDecimalGroup {
        heading,
        negative: NegativeSection {
            arguments: parse_optional_open_texts(sections.get("negative").copied(), tracker),
        },
        decimal: DecimalSection {
            arguments: parse_optional_open_texts(sections.get("decimal").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}
