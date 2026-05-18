"use client";

import { useState } from "react";
import { ArgumentList } from "./argument-list";
import type { GroupView, SectionView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";
import styles from "./group-card.module.css";
import sectionStyles from "./section-content.module.css";

type GroupCardProps = {
  anchorId: string;
  group: GroupView;
};

export function GroupCard({ anchorId, group }: GroupCardProps) {
  const [showDocumented, setShowDocumented] = useState(false);
  const headingTooltip = group.heading ?? undefined;
  const headingLatex = group.heading_latex
    ? capitalizeFirstRenderedLatexWord(group.heading_latex)
    : null;
  const hasHeading = headingLatex !== null;
  const hasDocumented = group.sections.some(isDocumentedSection);
  const visibleSections = showDocumented
    ? group.sections
    : group.sections.filter((section) => !isDocumentedSection(section));

  return (
    <section className={styles.card} id={anchorId}>
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
      {hasDocumented ? (
        <button
          aria-expanded={showDocumented}
          aria-label={
            showDocumented
              ? "Hide documented section"
              : "Show documented section"
          }
          className={styles.documentedToggle}
          onClick={() => setShowDocumented((value) => !value)}
          title={
            showDocumented
              ? "Hide documented section"
              : "Show documented section"
          }
          type="button"
        >
          <span className={styles.documentedToggleChevron} aria-hidden="true" />
        </button>
      ) : null}
    </section>
  );
}

function isDocumentedSection(section: SectionView): boolean {
  return section.label === "Documented";
}

function capitalizeFirstRenderedLatexWord(latex: string): string {
  for (let index = 0; index < latex.length; index += 1) {
    const textCommand = latexTextCommandAt(latex, index);

    if (!textCommand) {
      continue;
    }

    const scan = scanLatexTextGroup(latex, index + textCommand.length);

    if (scan.letterIndex !== null) {
      const letter = latex[scan.letterIndex].toUpperCase();

      return `${latex.slice(0, scan.letterIndex)}${letter}${latex.slice(scan.letterIndex + 1)}`;
    }

    index = scan.endIndex;
  }

  return latex;
}

function latexTextCommandAt(latex: string, index: number): string | null {
  for (const command of ["\\textrm{", "\\text{"]) {
    if (latex.startsWith(command, index)) {
      return command;
    }
  }

  return null;
}

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

function isAsciiLetter(value: string): boolean {
  return /^[A-Za-z]$/.test(value);
}
