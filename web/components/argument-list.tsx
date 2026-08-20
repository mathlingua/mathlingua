import { ArgumentView } from "../lib/types";
import { LatexRenderer } from "./latex-renderer";
import { MarkdownInline } from "./markdown-text";
import { MathLinguaInline } from "./mathlingua-inline";
import { TypeAnnotations } from "./type-annotations";
import styles from "./argument-list.module.css";
import sectionStyles from "./section-content.module.css";

/** Props for rendering the recursive list of section arguments. */
interface ArgumentListProps {
  /** Arguments nested under the current section. */
  arguments: ArgumentView[];
  /** Called when rendered math references another definition. */
  onReferenceClick?: (referenceKey: string) => void;
  /** Whether resolved expression types are visible. */
  showTypes?: boolean;
}

/** Renders formulation, text, and nested-group section arguments. */
export function ArgumentList({
  arguments: items,
  onReferenceClick,
  showTypes = false,
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
            <div className={styles.formulationBlock}>
              {argument.label ? (
                <div className={styles.labeledFormulation}>
                  {argument.latex ? (
                    <span
                      className={`${styles.formulationLine} ${styles.formulationLineLatex}`}
                    >
                      <LatexRenderer
                        latex={argument.latex}
                        onReferenceClick={onReferenceClick}
                      />
                    </span>
                  ) : (
                    <MathLinguaInline
                      className={styles.formulationLine}
                      text={argument.text}
                    />
                  )}
                  <span className={styles.formulationLabel}>
                    [{argument.label}]
                  </span>
                </div>
              ) : argument.latex ? (
                <span
                  className={`${styles.formulationLine} ${styles.formulationLineLatex}`}
                >
                  <LatexRenderer
                    latex={argument.latex}
                    onReferenceClick={onReferenceClick}
                  />
                </span>
              ) : (
                <MathLinguaInline
                  className={styles.formulationLine}
                  text={argument.text}
                />
              )}
              {showTypes && (argument.type_info?.length ?? 0) > 0 ? (
                <TypeAnnotations entries={argument.type_info ?? []} />
              ) : null}
            </div>
          ) : null}
          {argument.kind === "text" ? (
            argument.latex ? (
              <span
                className={`${styles.textLine} ${styles.formulationLineLatex}`}
              >
                <LatexRenderer
                  latex={argument.latex}
                  onReferenceClick={onReferenceClick}
                />
              </span>
            ) : (
              <p className={styles.textLine}>{argument.text}</p>
            )
          ) : null}
          {argument.kind === "reference" ? (
            argument.href && isSafeReferenceHref(argument.href) ? (
              <a
                className={styles.referenceLink}
                href={argument.href}
                rel="noreferrer"
                target="_blank"
                title={argument.source}
              >
                <MarkdownInline text={argument.text} />
              </a>
            ) : (
              <span className={styles.referenceText} title={argument.source}>
                <MarkdownInline text={argument.text} />
              </span>
            )
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
        </li>
      ))}
    </ul>
  );
}

function isSafeReferenceHref(href: string): boolean {
  return /^https?:\/\//i.test(href);
}
