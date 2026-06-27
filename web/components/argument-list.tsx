import { ArgumentView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";
import styles from "./argument-list.module.css";
import sectionStyles from "./section-content.module.css";

/** Props for rendering the recursive list of section arguments. */
interface ArgumentListProps {
  /** Arguments nested under the current section. */
  arguments: ArgumentView[];
  /** Called when rendered math references another definition. */
  onReferenceClick?: (referenceKey: string) => void;
}

/** Renders formulation, text, and nested-group section arguments. */
export function ArgumentList({
  arguments: items,
  onReferenceClick,
}: ArgumentListProps) {
  return (
    <ul className={styles.list}>
      {items.map((argument, index) => (
        <li
          className={
            argument.kind === "group"
              ? `${styles.item} ${styles.itemGroup}`
              : styles.item
          }
          key={`${argument.kind}-${index}`}
        >
          {argument.kind === "formulation" ? (
            argument.latex ? (
              <span
                className={`${styles.formulationLine} ${styles.formulationLineLatex}`}
              >
                <LatexRenderer
                  latex={argument.latex}
                  onReferenceClick={onReferenceClick}
                />
              </span>
            ) : (
              <code className={styles.formulationLine}>{argument.text}</code>
            )
          ) : null}
          {argument.kind === "text" ? (
            <p className={styles.textLine}>{argument.text}</p>
          ) : null}
          {argument.kind === "group" ? (
            <div className={styles.nestedGroup}>
              {argument.heading ? (
                <p className={styles.nestedHeading}>[{argument.heading}]</p>
              ) : null}
              {argument.sections.map((section, sectionIndex) => (
                <section
                  className={styles.nestedSection}
                  key={`${section.label}-${sectionIndex}`}
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
                        <code className={sectionStyles.inlineArgument}>
                          {section.inline_argument}
                        </code>
                      )
                    ) : null}
                  </div>
                  {section.arguments.length > 0 ? (
                    <ArgumentList
                      arguments={section.arguments}
                      onReferenceClick={onReferenceClick}
                    />
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
