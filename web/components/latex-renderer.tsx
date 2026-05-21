"use client";

import katex from "katex";
import styles from "./latex-renderer.module.css";

/** Props for rendering one inline LaTeX fragment. */
interface LatexRendererProps {
  /** LaTeX emitted by the Rust view renderer. */
  latex: string;
}

/** Renders trusted backend-produced LaTeX through KaTeX. */
export function LatexRenderer({ latex }: LatexRendererProps) {
  const html = katex.renderToString(latex, {
    displayMode: false,
    throwOnError: false,
  });

  return (
    <span
      className={styles.renderer}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
