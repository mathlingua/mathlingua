"use client";

import { useState } from "react";
import { ArgumentList } from "./argument-list";
import type { GroupView, SectionView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";
import { MathLinguaSource } from "./mathlingua-source";
import { MathLinguaInline } from "./mathlingua-inline";
import { MarkdownInline, MarkdownText } from "./markdown-text";
import { TypeAnnotations } from "./type-annotations";
import styles from "./group-card.module.css";
import sectionStyles from "./section-content.module.css";

/** Props for rendering one top-level MathLingua group card. */
interface GroupCardProps {
  /** Stable DOM id used for linking directly to the group. */
  anchorId: string;
  /** Serialized group data emitted by the Rust view builder. */
  group: GroupView;
  /** Called when rendered math references another definition. */
  onReferenceClick?: (referenceKey: string) => void;
  /** Optional close action shown for inline definition cards. */
  onClose?: () => void;
  /** Whether resolved expression types are visible. */
  showTypes?: boolean;
}

/**
 * Renders one MathLingua group and hides optional support sections until the
 * reader asks for them.
 */
export function GroupCard({
  anchorId,
  group,
  onReferenceClick,
  onClose,
  showTypes = false,
}: GroupCardProps) {
  const [showSupportSections, setShowSupportSections] = useState(false);
  const [showSource, setShowSource] = useState(false);
  const [showProofSource, setShowProofSource] = useState(false);
  const headingTooltip = group.heading ?? undefined;
  const headingLatex = group.heading_latex
    ? capitalizeLeadingRenderedLatexWord(group.heading_latex)
    : null;
  const resourceCard = buildResourceCard(group);
  const parameterDestructurings = group.parameter_destructurings ?? [];
  const hasHeading = resourceCard !== null || headingLatex !== null;
  const bodyText = group.body_text?.trim() ? group.body_text : null;
  const proofText = group.proof_text?.trim() ? group.proof_text : null;
  const proofSource = group.proof_source?.trim() ? group.proof_source : null;
  const hasSupportSections =
    group.sections.some(
      (section) =>
        isSupportSection(group.kind, section) &&
        !isCardBodySection(group.kind, section),
    ) || resourceCardHasHiddenFields(resourceCard);
  const frontFaceClass = `${styles.cardFace} ${styles.cardFront}${
    onClose ? ` ${styles.cardFaceClosable}` : ""
  }${showSource ? ` ${styles.cardFaceInactive}` : ""}`;
  const backFaceClass = `${styles.cardFace} ${styles.cardBack}${
    onClose ? ` ${styles.cardFaceClosable}` : ""
  }${showSource ? "" : ` ${styles.cardFaceInactive}`}`;
  const visibleSections = showSupportSections
    ? group.sections.filter(
        (section) => !isCardBodySection(group.kind, section),
      )
    : group.sections.filter(
        (section) =>
          !isSupportSection(group.kind, section) &&
          !isCardBodySection(group.kind, section),
      );
  const hasVisibleSections = visibleSections.length > 0;

  return (
    <section className={styles.card} id={anchorId}>
      <div
        className={
          showSource
            ? `${styles.cardStage} ${styles.cardStageFlipped}`
            : styles.cardStage
        }
      >
        <article aria-hidden={showSource} className={frontFaceClass}>
          <button
            aria-label="Show MathLingua source"
            className={`${styles.iconToggle} ${styles.sourceToggle}`}
            onClick={() => setShowSource(true)}
            tabIndex={showSource ? -1 : 0}
            title="Show MathLingua source"
            type="button"
          >
            <CodeIcon />
          </button>
          {onClose ? (
            <button
              aria-label="Close definition"
              className={`${styles.iconToggle} ${styles.closeToggle}`}
              onClick={onClose}
              tabIndex={showSource ? -1 : 0}
              title="Close definition"
              type="button"
            >
              <CloseIcon />
            </button>
          ) : null}
          {hasHeading ? (
            <header className={styles.header}>
              <h3
                className={
                  resourceCard
                    ? `${styles.heading} ${styles.resourceHeading}`
                    : `${styles.heading} ${styles.headingLatex}`
                }
                title={resourceCard?.href ?? headingTooltip}
              >
                {resourceCard ? (
                  resourceCard.href ? (
                    <a
                      className={styles.resourceTitleLink}
                      href={resourceCard.href}
                      rel="noreferrer"
                      target="_blank"
                    >
                      <MarkdownInline text={resourceCard.title} />
                    </a>
                  ) : (
                    <MarkdownInline text={resourceCard.title} />
                  )
                ) : headingLatex ? (
                  <LatexRenderer
                    latex={headingLatex}
                    onReferenceClick={onReferenceClick}
                  />
                ) : null}
              </h3>
              {parameterDestructurings.length > 0 ? (
                <div className={styles.parameterDestructurings}>
                  {parameterDestructurings.map((latex, index) => (
                    <span className={styles.parameterDestructuring} key={index}>
                      <LatexRenderer
                        latex={latex}
                        onReferenceClick={onReferenceClick}
                      />
                    </span>
                  ))}
                </div>
              ) : null}
            </header>
          ) : null}
          {bodyText ? (
            <div className={styles.bodyText}>
              <MarkdownText
                onReferenceClick={onReferenceClick}
                text={bodyText}
              />
            </div>
          ) : null}
          {resourceCard ? (
            <ResourceCardDetails
              card={resourceCard}
              showHiddenFields={showSupportSections}
            />
          ) : null}
          {hasVisibleSections ? (
            <div
              className={
                hasHeading || bodyText || resourceCard
                  ? styles.sectionStack
                  : `${styles.sectionStack} ${styles.sectionStackFlush}`
              }
            >
              {visibleSections.map((section, index) => (
                <section
                  className={styles.sectionBlock}
                  key={`${section.label}-${index}`}
                >
                  <div className={sectionStyles.sectionLabelRow}>
                    <span className={sectionStyles.sectionLabel}>
                      {section.label}
                    </span>
                    {section.inline_argument ? (
                      section.inline_latex ? (
                        <span
                          className={`${sectionStyles.inlineArgument} ${sectionStyles.inlineArgumentLatex}`}
                        >
                          <LatexRenderer
                            latex={section.inline_latex}
                            onReferenceClick={onReferenceClick}
                          />
                        </span>
                      ) : (
                        <MathLinguaInline
                          className={sectionStyles.inlineArgument}
                          text={section.inline_argument}
                        />
                      )
                    ) : null}
                  </div>
                  {showTypes && (section.inline_type_info?.length ?? 0) > 0 ? (
                    <TypeAnnotations entries={section.inline_type_info ?? []} />
                  ) : null}
                  {section.arguments.length > 0 ? (
                    <ArgumentList
                      arguments={section.arguments}
                      onReferenceClick={onReferenceClick}
                      showTypes={showTypes}
                    />
                  ) : null}
                </section>
              ))}
            </div>
          ) : null}
          {hasSupportSections ? (
            <button
              aria-expanded={showSupportSections}
              aria-label={
                showSupportSections
                  ? "Hide supporting sections"
                  : "Show supporting sections"
              }
              className={styles.documentedToggle}
              onClick={() => setShowSupportSections((value) => !value)}
              tabIndex={showSource ? -1 : 0}
              title={
                showSupportSections
                  ? "Hide supporting sections"
                  : "Show supporting sections"
              }
              type="button"
            >
              <span
                className={styles.documentedToggleChevron}
                aria-hidden="true"
              />
            </button>
          ) : null}
        </article>
        <article aria-hidden={!showSource} className={backFaceClass}>
          <button
            aria-label="Show rendered entry"
            className={`${styles.iconToggle} ${styles.sourceToggle}`}
            onClick={() => setShowSource(false)}
            tabIndex={showSource ? 0 : -1}
            title="Show rendered entry"
            type="button"
          >
            <CardIcon />
          </button>
          {onClose ? (
            <button
              aria-label="Close definition"
              className={`${styles.iconToggle} ${styles.closeToggle}`}
              onClick={onClose}
              tabIndex={showSource ? 0 : -1}
              title="Close definition"
              type="button"
            >
              <CloseIcon />
            </button>
          ) : null}
          <MathLinguaSource
            onReferenceClick={onReferenceClick}
            source={group.source}
          />
        </article>
      </div>
      {proofText ? (
        <article className={styles.proof}>
          <header className={styles.proofHeader}>
            <h4 className={styles.proofHeading}>Proof</h4>
            {proofSource ? (
              <button
                aria-label={
                  showProofSource
                    ? "Show rendered proof"
                    : "Show proof MathLingua source"
                }
                aria-pressed={showProofSource}
                className={`${styles.iconToggle} ${styles.proofSourceToggle}`}
                onClick={() => setShowProofSource((value) => !value)}
                title={
                  showProofSource
                    ? "Show rendered proof"
                    : "Show proof MathLingua source"
                }
                type="button"
              >
                {showProofSource ? <CardIcon /> : <CodeIcon />}
              </button>
            ) : null}
          </header>
          {showProofSource && proofSource ? (
            <div className={styles.proofSource}>
              <MathLinguaSource
                onReferenceClick={onReferenceClick}
                source={proofSource}
              />
            </div>
          ) : (
            <div className={styles.proofContent}>
              <div className={styles.proofBody}>
                <MarkdownText
                  onReferenceClick={onReferenceClick}
                  text={proofText}
                />
              </div>
              <span aria-label="End of proof" className={styles.proofEnd}>
                <LatexRenderer latex={"\\Box"} />
              </span>
            </div>
          )}
        </article>
      ) : null}
    </section>
  );
}

function CloseIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.sourceToggleIcon}
      focusable="false"
      viewBox="0 0 24 24"
    >
      <path d="M6 6l12 12" />
      <path d="M18 6 6 18" />
    </svg>
  );
}

function CodeIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.sourceToggleIcon}
      focusable="false"
      viewBox="0 0 24 24"
    >
      <path d="m9 8-4 4 4 4" />
      <path d="m15 8 4 4-4 4" />
      <path d="m13 5-2 14" />
    </svg>
  );
}

function CardIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.sourceToggleIcon}
      focusable="false"
      viewBox="0 0 24 24"
    >
      <path d="M5 6h14v12H5z" />
      <path d="M8 10h8" />
      <path d="M8 14h5" />
    </svg>
  );
}

/** Returns true for support sections hidden behind the card expander. */
function isSupportSection(groupKind: string, section: SectionView): boolean {
  // An item-level `Writing:` section only tunes how identifiers render, so it is
  // support content. The collection-wide `Writing:` group's leading section
  // carries the group's whole point, so it stays visible.
  if (section.label === "Writing") {
    return groupKind !== "Writing";
  }

  return (
    section.label === "Documented" ||
    section.label === "References" ||
    section.label === "Provides" ||
    section.label === "Justification" ||
    section.label === "Id"
  );
}

/** The four opaque `Text*` placeholder group kinds. */
function isTextItemKind(kind: string): boolean {
  return (
    kind === "TextTheorem" ||
    kind === "TextAxiom" ||
    kind === "TextConjecture" ||
    kind === "TextDefinition"
  );
}

/** Sections rendered as first-class card title/body content. */
function isCardBodySection(groupKind: string, section: SectionView): boolean {
  return (
    (groupKind === "Person" &&
      (section.label === "Person" || section.label === "biography")) ||
    (groupKind === "Resource" && section.label === "Resource") ||
    // A `Text*` placeholder's body section renders as the card's markdown body.
    (isTextItemKind(groupKind) && section.label === groupKind)
  );
}

interface ResourceCardData {
  title: string;
  href: string | null;
  fields: ResourceField[];
}

interface ResourceField {
  label: string;
  values: string[];
  kind: "inline" | "link" | "description";
  hidden: boolean;
}

function ResourceCardDetails({
  card,
  showHiddenFields,
}: {
  card: ResourceCardData;
  showHiddenFields: boolean;
}) {
  const descriptionFields = card.fields.filter(
    (field) => field.kind === "description",
  );
  const metadataFields = card.fields.filter(
    (field) =>
      field.kind !== "description" && (!field.hidden || showHiddenFields),
  );

  if (descriptionFields.length === 0 && metadataFields.length === 0) {
    return null;
  }

  return (
    <div className={styles.resourceBody}>
      {metadataFields.length > 0 ? (
        <dl className={styles.resourceMeta}>
          {metadataFields.map((field) => (
            <div className={styles.resourceMetaRow} key={field.label}>
              <dt className={styles.resourceMetaLabel}>{field.label}</dt>
              <dd className={styles.resourceMetaValue}>
                {field.values.map((value, index) => (
                  <span className={styles.resourceMetaItem} key={index}>
                    {field.kind === "link" && isSafeResourceHref(value) ? (
                      <a
                        className={styles.resourceDataLink}
                        href={value}
                        rel="noreferrer"
                        target="_blank"
                      >
                        {value}
                      </a>
                    ) : (
                      <MarkdownInline text={value} />
                    )}
                  </span>
                ))}
              </dd>
            </div>
          ))}
        </dl>
      ) : null}
      {descriptionFields.map((field) => (
        <div className={styles.resourceDescription} key={field.label}>
          {field.values.map((value, index) => (
            <MarkdownText key={index} text={value} />
          ))}
        </div>
      ))}
    </div>
  );
}

function buildResourceCard(group: GroupView): ResourceCardData | null {
  if (group.kind !== "Resource") {
    return null;
  }

  const resourceSection = group.sections.find(
    (section) => section.label === "Resource",
  );

  if (!resourceSection) {
    return null;
  }

  const rawFields = new Map<string, string[]>();

  for (const argument of resourceSection.arguments) {
    if (argument.kind !== "group") {
      continue;
    }

    for (const section of argument.sections) {
      const values = sectionTextValues(section);

      if (values.length === 0) {
        continue;
      }

      rawFields.set(section.label, [
        ...(rawFields.get(section.label) ?? []),
        ...values,
      ]);
    }
  }

  const title =
    rawFields.get("title")?.[0] ??
    readableResourceHeading(group.heading) ??
    "Resource";
  const href = rawFields.get("url")?.find(isSafeResourceHref) ?? null;
  const fields: ResourceField[] = [];

  for (const label of RESOURCE_FIELD_ORDER) {
    if (label === "title") {
      continue;
    }

    const values = rawFields.get(label);

    if (!values || values.length === 0) {
      continue;
    }

    fields.push({
      hidden: label === "url",
      label: resourceFieldLabel(label),
      values,
      kind:
        label === "description"
          ? "description"
          : label === "url" || label === "homepage"
            ? "link"
            : "inline",
    });
  }

  for (const [label, values] of rawFields) {
    if (label === "title" || RESOURCE_FIELD_ORDER.includes(label)) {
      continue;
    }

    fields.push({
      hidden: false,
      label: resourceFieldLabel(label),
      values,
      kind: "inline",
    });
  }

  return { fields, href, title };
}

function resourceCardHasHiddenFields(card: ResourceCardData | null): boolean {
  return card?.fields.some((field) => field.hidden) ?? false;
}

const RESOURCE_FIELD_ORDER = [
  "title",
  "author",
  "editor",
  "type",
  "journal",
  "publisher",
  "institution",
  "edition",
  "volume",
  "month",
  "year",
  "offset",
  "url",
  "homepage",
  "description",
];

function sectionTextValues(section: SectionView): string[] {
  const values: string[] = [];

  if (section.inline_argument) {
    values.push(stripQuotedText(section.inline_argument));
  }

  for (const argument of section.arguments) {
    if (argument.kind === "text" || argument.kind === "formulation") {
      values.push(argument.text);
    }
  }

  return values.map((value) => value.trim()).filter(Boolean);
}

function stripQuotedText(text: string): string {
  const trimmed = text.trim();

  if (trimmed.startsWith('"') && trimmed.endsWith('"') && trimmed.length >= 2) {
    return trimmed.slice(1, -1);
  }

  return trimmed;
}

function readableResourceHeading(heading: string | null): string | null {
  if (!heading?.startsWith("$")) {
    return null;
  }

  const words = heading
    .slice(1)
    .split(".")
    .filter(Boolean)
    .map((word) => word.replace(/_/g, " "));

  if (words.length === 0) {
    return null;
  }

  return words.map(capitalizeWord).join(" ");
}

function capitalizeWord(word: string): string {
  return word.replace(/[A-Za-z]/, (letter) => letter.toUpperCase());
}

function resourceFieldLabel(label: string): string {
  if (label === "url") {
    return "URL";
  }

  return label
    .replace(/_/g, " ")
    .replace(/[A-Za-z]/, (letter) => letter.toUpperCase());
}

function isSafeResourceHref(href: string): boolean {
  return /^https?:\/\//i.test(href);
}

/**
 * Capitalizes a heading that starts with rendered text-mode LaTeX.
 *
 * The backend preserves documented `called:` capitalization, while card
 * headings read better when an all-prose title starts uppercase. Mixed
 * math/prose titles should keep their original prose casing.
 */
function capitalizeLeadingRenderedLatexWord(latex: string): string {
  let index = 0;

  while (index < latex.length && /\s/.test(latex[index])) {
    index += 1;
  }

  const textCommand = latexTextCommandAt(latex, index);

  if (!textCommand) {
    return latex;
  }

  const scan = scanLatexTextGroup(latex, index + textCommand.length);

  if (scan.letterIndex !== null) {
    const letter = latex[scan.letterIndex].toUpperCase();

    return `${latex.slice(0, scan.letterIndex)}${letter}${latex.slice(scan.letterIndex + 1)}`;
  }

  return latex;
}

/** Detects a text-mode LaTeX command at the given string index. */
function latexTextCommandAt(latex: string, index: number): string | null {
  for (const command of ["\\textrm{", "\\text{"]) {
    if (latex.startsWith(command, index)) {
      return command;
    }
  }

  return null;
}

/** Scans one LaTeX text group for the first renderable ASCII letter. */
function scanLatexTextGroup(
  latex: string,
  startIndex: number,
): { letterIndex: number | null; endIndex: number } {
  let depth = 1;

  for (let index = startIndex; index < latex.length; index += 1) {
    const char = latex[index];

    if (char === "\\") {
      index = skipLatexCommand(latex, index);
      continue;
    }

    if (char === "{") {
      depth += 1;
      continue;
    }

    if (char === "}") {
      depth -= 1;

      if (depth === 0) {
        return { letterIndex: null, endIndex: index };
      }

      continue;
    }

    if (isAsciiLetter(char)) {
      return { letterIndex: index, endIndex: index };
    }
  }

  return { letterIndex: null, endIndex: latex.length };
}

/** Skips over a LaTeX command name so nested commands are not capitalized. */
function skipLatexCommand(latex: string, startIndex: number): number {
  let index = startIndex + 1;

  if (index >= latex.length || !isAsciiLetter(latex[index])) {
    return index;
  }

  while (index + 1 < latex.length && isAsciiLetter(latex[index + 1])) {
    index += 1;
  }

  return index;
}

/** Returns true when a string contains exactly one ASCII letter. */
function isAsciiLetter(value: string): boolean {
  return /^[A-Za-z]$/.test(value);
}
