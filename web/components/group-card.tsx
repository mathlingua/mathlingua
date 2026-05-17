"use client";

import { useState } from "react";
import { ArgumentList } from "./argument-list";
import { formatGroupHeading } from "../lib/presenter";
import type { GroupView, SectionView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";

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
  const hasDocumented = group.sections.some(isDocumentedSection);
  const visibleSections = showDocumented
    ? group.sections
    : group.sections.filter((section) => !isDocumentedSection(section));

  return (
    <section className="group-card" id={anchorId}>
      <header className="group-header">
        <h3
          className={`group-heading${headingLatex ? " group-heading--latex" : ""}`}
          title={headingTooltip}
        >
          {headingLatex ? (
            <LatexRenderer latex={headingLatex} />
          ) : (
            formatGroupHeading(group)
          )}
        </h3>
      </header>
      <div className="section-stack">
        {visibleSections.map((section, index) => (
          <section className="section-block" key={`${section.label}-${index}`}>
            <div className="section-label-row">
              <span className="section-label">{section.label}</span>
              {section.inline_argument ? (
                section.inline_latex ? (
                  <span className="inline-argument inline-argument--latex">
                    <LatexRenderer latex={section.inline_latex} />
                  </span>
                ) : (
                  <code className="inline-argument">
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
          className="documented-toggle"
          onClick={() => setShowDocumented((value) => !value)}
          title={
            showDocumented
              ? "Hide documented section"
              : "Show documented section"
          }
          type="button"
        >
          <span className="documented-toggle__chevron" aria-hidden="true" />
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
