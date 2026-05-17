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
  const hasDocumented = group.sections.some(isDocumentedSection);
  const visibleSections = showDocumented
    ? group.sections
    : group.sections.filter((section) => !isDocumentedSection(section));

  return (
    <section className="group-card" id={anchorId}>
      <header className="group-header">
        <h3
          className={`group-heading${group.heading_latex ? " group-heading--latex" : ""}`}
          title={headingTooltip}
        >
          {group.heading_latex ? (
            <LatexRenderer latex={group.heading_latex} />
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
