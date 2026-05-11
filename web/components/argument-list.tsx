import { ArgumentView } from "../lib/types";

type ArgumentListProps = {
  arguments: ArgumentView[];
};

export function ArgumentList({ arguments: items }: ArgumentListProps) {
  return (
    <ul className="argument-list">
      {items.map((argument, index) => (
        <li className="argument-item" key={`${argument.kind}-${index}`}>
          {argument.kind === "formulation" ? (
            <code className="formulation-line">{argument.text}</code>
          ) : null}
          {argument.kind === "text" ? (
            <p className="text-line">{argument.text}</p>
          ) : null}
          {argument.kind === "group" ? (
            <div className="nested-group">
              {argument.heading ? (
                <p className="nested-heading">[{argument.heading}]</p>
              ) : null}
              {argument.sections.map((section, sectionIndex) => (
                <section
                  className="nested-section"
                  key={`${section.label}-${sectionIndex}`}
                >
                  <div className="section-label-row">
                    <span className="section-label">{section.label}</span>
                    {section.inline_argument ? (
                      <code className="inline-argument">
                        {section.inline_argument}
                      </code>
                    ) : null}
                  </div>
                  {section.arguments.length > 0 ? (
                    <ArgumentList arguments={section.arguments} />
                  ) : null}
                </section>
              ))}
            </div>
          ) : null}
        </li>
      ))}
    </ul>
  );
}
