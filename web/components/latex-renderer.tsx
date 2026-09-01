"use client";

import katex from "katex";
import type { MouseEvent } from "react";
import styles from "./latex-renderer.module.css";

/** Props for rendering one inline LaTeX fragment. */
interface LatexRendererProps {
  /** LaTeX emitted by the Rust view renderer. */
  latex: string;
  /** Called when a linked MathLingua definition reference is clicked. */
  onReferenceClick?: (referenceKey: string) => void;
}

/** Renders trusted backend-produced LaTeX through KaTeX. */
export function LatexRenderer({ latex, onReferenceClick }: LatexRendererProps) {
  const html = katex.renderToString(latex, {
    displayMode: false,
    strict: "ignore",
    throwOnError: false,
    trust: allowMathLinguaCommands,
  });

  const handleClick = (event: MouseEvent<HTMLSpanElement>) => {
    if (!onReferenceClick) {
      return;
    }

    const target =
      event.target instanceof Element
        ? event.target.closest("[data-mlg-ref]")
        : null;
    if (!target || !event.currentTarget.contains(target)) {
      return;
    }

    const referenceKey = target.getAttribute("data-mlg-ref");
    if (!referenceKey) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    onReferenceClick(referenceKey);
  };

  return (
    <span
      className={
        onReferenceClick
          ? `${styles.renderer} ${styles.rendererInteractive}`
          : styles.renderer
      }
      onClick={handleClick}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

/** The one `\htmlClass` the backend emits: the muted written form of a card title. */
const TITLE_WRITTEN_CLASS = "mlg-title-written";

/**
 * Trusts only the specific HTML commands the backend renderer emits: a
 * `data-mlg-ref` marker for a linked definition reference, and the single class
 * that mutes the written half of a two-part card title. Everything else is
 * rejected so arbitrary markup in a formulation cannot reach the page.
 */
export function allowMathLinguaCommands(context: {
  command?: string;
  attributes?: Record<string, string>;
  class?: string;
}): boolean {
  if (context.command === "\\htmlClass") {
    return context.class === TITLE_WRITTEN_CLASS;
  }

  if (context.command === "\\htmlData" && context.attributes) {
    const attributes = Object.keys(context.attributes);
    return (
      attributes.length === 1 &&
      attributes[0] === "data-mlg-ref" &&
      /^[0-9a-f]+$/.test(context.attributes["data-mlg-ref"] ?? "")
    );
  }

  return false;
}
