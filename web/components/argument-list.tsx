import { ArgumentView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";

type ArgumentListProps = {
  arguments: ArgumentView[];
};

export function ArgumentList({ arguments: items }: ArgumentListProps) {
  return (
    <ul className="argument-list">
      {items.map((argument, index) => (
        <li
          className={`argument-item argument-item--${argument.kind}`}
          key={`${argument.kind}-${index}`}
        >
          {argument.kind === "formulation" ? (
            argument.latex ? (
              <span className="formulation-line formulation-line--latex">
                <LatexRenderer latex={argument.latex} />
              </span>
            ) : (
              <code className="formulation-line">{argument.text}</code>
            )
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
          ) : null}
        </li>
      ))}
    </ul>
  );
}
