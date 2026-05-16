import { ArgumentList } from "./argument-list";
import { formatGroupHeading } from "../lib/presenter";
import { GroupView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";

type GroupCardProps = {
  anchorId: string;
  group: GroupView;
};

export function GroupCard({ anchorId, group }: GroupCardProps) {
  return (
    <section className="group-card" id={anchorId}>
      <header className="group-header">
        <h3 className="group-heading">{formatGroupHeading(group)}</h3>
      </header>
      <div className="section-stack">
        {group.sections.map((section, index) => (
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
    </section>
  );
}
