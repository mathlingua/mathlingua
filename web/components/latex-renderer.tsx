"use client";

import katex from "katex";
import styles from "./latex-renderer.module.css";

type LatexRendererProps = {
  latex: string;
};

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
