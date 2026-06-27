"use client";

import { useState } from "react";
import { ArgumentList } from "./argument-list";
import type { GroupView, SectionView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";
import { MathLinguaSource } from "./mathlingua-source";
import styles from "./group-card.module.css";
import sectionStyles from "./section-content.module.css";

/** Props for rendering one top-level MathLingua group card. */
interface GroupCardProps {
  /** Stable DOM id used for linking directly to the group. */
  anchorId: string;
  /** Serialized group data emitted by the Rust view builder. */
  group: GroupView;
}

/**
 * Renders one MathLingua group and hides optional support sections until the
 * reader asks for them.
 */
export function GroupCard({ anchorId, group }: GroupCardProps) {
  const [showSupportSections, setShowSupportSections] = useState(false);
  const [showSource, setShowSource] = useState(false);
  const headingTooltip = group.heading ?? undefined;
  const headingLatex = group.heading_latex
    ? capitalizeLeadingRenderedLatexWord(group.heading_latex)
    : null;
  const hasHeading = headingLatex !== null;
  const hasSupportSections = group.sections.some(isSupportSection);
  const visibleSections = showSupportSections
    ? group.sections
    : group.sections.filter((section) => !isSupportSection(section));

  return (
    <section className={styles.card} id={anchorId}>
      <div
        className={
          showSource
            ? `${styles.cardStage} ${styles.cardStageFlipped}`
            : styles.cardStage
        }
      >
        <article
          aria-hidden={showSource}
          className={
            showSource
              ? `${styles.cardFace} ${styles.cardFront} ${styles.cardFaceInactive}`
              : `${styles.cardFace} ${styles.cardFront}`
          }
        >
          <button
            aria-label="Show MathLingua source"
            className={styles.sourceToggle}
            onClick={() => setShowSource(true)}
            tabIndex={showSource ? -1 : 0}
            title="Show MathLingua source"
            type="button"
          >
            <CodeIcon />
          </button>
          {hasHeading ? (
            <header className={styles.header}>
              <h3
                className={`${styles.heading} ${styles.headingLatex}`}
                title={headingTooltip}
              >
                <LatexRenderer latex={headingLatex} />
              </h3>
            </header>
          ) : null}
          <div
            className={
              hasHeading
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
                        <LatexRenderer latex={section.inline_latex} />
                      </span>
                    ) : (
                      <code className={sectionStyles.inlineArgument}>
                        {section.inline_argument}
                      </code>
                    )
                  ) : null}
                </div>
                {section.arguments.length > 0 ? (
                  <ArgumentList arguments={section.arguments} />
                ) : null}
              </section>
            ))}
          </div>
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
        <article
          aria-hidden={!showSource}
          className={
            showSource
              ? `${styles.cardFace} ${styles.cardBack}`
              : `${styles.cardFace} ${styles.cardBack} ${styles.cardFaceInactive}`
          }
        >
          <button
            aria-label="Show rendered entry"
            className={styles.sourceToggle}
            onClick={() => setShowSource(false)}
            tabIndex={showSource ? 0 : -1}
            title="Show rendered entry"
            type="button"
          >
            <CardIcon />
          </button>
          <MathLinguaSource source={group.source} />
        </article>
      </div>
    </section>
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
function isSupportSection(section: SectionView): boolean {
  return section.label === "Documented" || section.label === "Provides";
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
