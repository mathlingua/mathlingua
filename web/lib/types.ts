export type CollectionView = {
  title: string;
  files: FileView[];
};

export type FileView = {
  path: string;
  items: GroupView[];
};

export type GroupView = {
  kind: string;
  heading: string | null;
  sections: SectionView[];
};

export type SectionView = {
  label: string;
  inline_argument: string | null;
  arguments: ArgumentView[];
};

export type ArgumentView =
  | {
      kind: "formulation";
      text: string;
    }
  | {
      kind: "text";
      text: string;
    }
  | {
      kind: "group";
      heading: string | null;
      sections: SectionView[];
    };
