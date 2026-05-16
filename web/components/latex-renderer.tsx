"use client";

import katex from "katex";

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
      className="latex-renderer"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
