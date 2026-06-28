/** Serialized view model emitted by the Rust backend for an entire collection. */
export type CollectionView = {
  /** Human-readable collection title derived from the collection root. */
  title: string;
  /** Renderable directories, already ordered by the backend. */
  directories: DirectoryView[];
  /** Renderable source files, already ordered by the backend. */
  files: FileView[];
};

/** Serialized view model for one MathLingua source directory. */
export type DirectoryView = {
  /** Directory path relative to the collection root when possible. */
  path: string;
  /** Optional display title supplied by a directory toc file. */
  title: string | null;
};

/** Serialized view model for one MathLingua source file. */
export type FileView = {
  /** File path relative to the collection root when possible. */
  path: string;
  /** Optional display title supplied by a directory toc file. */
  title: string | null;
  /** Top-level groups rendered from the file. */
  items: GroupView[];
};

/** Serialized view model for one top-level MathLingua group card. */
export type GroupView = {
  /** Structural group kind, such as `Describes`, `Refines`, or `Theorem`. */
  kind: string;
  /** Reference keys that resolve to this group when rendered math is clicked. */
  definition_keys: string[];
  /** Raw bracket heading text, if the source group had one. */
  heading: string | null;
  /** Backend-rendered LaTeX title for the group card, if available. */
  heading_latex: string | null;
  /** Direct page content for document headings/prose instead of card content. */
  page: PageView | null;
  /** Original MathLingua source for this top-level group. */
  source: string;
  /** Rendered sections belonging to the group. */
  sections: SectionView[];
};

/** Direct document-flow item rendered outside cards. */
export type PageView = {
  /** Page item kind, such as `Title`, `SectionTitle`, `SubsectionTitle`, or `Text`. */
  kind: string;
  /** Quote-stripped text content. */
  text: string;
};

/** Serialized view model for one labeled section inside a group. */
export type SectionView = {
  /** Section label without the trailing colon. */
  label: string;
  /** Raw inline argument after the section label, if one was present. */
  inline_argument: string | null;
  /** Backend-rendered LaTeX for the inline argument, if available. */
  inline_latex: string | null;
  /** Block arguments nested under this section. */
  arguments: ArgumentView[];
};

/** Serialized representation of a section argument. */
export type ArgumentView =
  | {
      /** Formulation arguments can render as LaTeX or fall back to source text. */
      kind: "formulation";
      /** Raw formulation text from the source file. */
      text: string;
      /** Backend-rendered LaTeX when parsing and rendering succeeded. */
      latex: string | null;
    }
  | {
      /** Plain text arguments render as prose. */
      kind: "text";
      /** Text content after source quoting has been removed. */
      text: string;
    }
  | {
      /** Nested group arguments recursively contain rendered sections. */
      kind: "group";
      /** Raw nested group heading, if present. */
      heading: string | null;
      /** Rendered sections inside the nested group. */
      sections: SectionView[];
    };
