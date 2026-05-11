import { ArgumentList } from "./argument-list";
import { formatGroupTitle } from "../lib/presenter";
import { GroupView } from "../lib/types";

type GroupCardProps = {
  anchorId: string;
  group: GroupView;
};

export function GroupCard({ anchorId, group }: GroupCardProps) {
  return (
    <section className="group-card" id={anchorId}>
      <header className="group-header">
        <div>
          <p className="group-kind">{group.kind}</p>
          <h3>{formatGroupTitle(group)}</h3>
          {group.heading ? <p className="group-heading-meta">{group.heading}</p> : null}
        </div>
        <span aria-hidden="true" className="group-menu">
          ≡
        </span>
      </header>
      <div className="section-stack">
        {group.sections.map((section, index) => (
          <section className="section-block" key={`${section.label}-${index}`}>
            <div className="section-label-row">
              <span className="section-label">{section.label}</span>
              {section.inline_argument ? (
                <code className="inline-argument">{section.inline_argument}</code>
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
