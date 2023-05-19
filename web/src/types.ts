
export type TextBlockType = "TextBlockType";
export type DocumentType = "DocumentType";
export type GroupType = "GroupType";
export type SectionType = "SectionType";
export type ArgumentType = "ArgumentType";
export type TextArgumentDataKind = "TextArgumentDataKind";
export type FormulationArgumentDataKind = "FormulationArgumentDataKind";
export type ArgumentTextArgumentDataKind = "ArgumentTextArgumentDataKind";

export interface Position {
	Offset: number;
	Row: number;
	Column: number;
}

export type DiagnosticType = string;
export type DiagnosticOrigin = string;

export interface Diagnostic {
	Type: DiagnosticType;
	Origin: DiagnosticOrigin;
	Message: string;
	Position: Position;
}

export interface PathLabelPair {
  Path: string;
  Label: string;
}

export interface PathsResponse {
	Error: string;
	Paths: PathLabelPair[] | null;
}

export interface PageResponse {
	Error: string;
	Diagnostics: Diagnostic[] | null;
	Document: Document;
}

export interface MetaData {
  Start: Position;
}

export interface TextBlock {
	Type: TextBlockType;
	Text: string;
	MetaData: MetaData;
}

export type TopLevelNodeKind = TextBlock | Group | null;

export interface Document {
	Type: DocumentType;
	Nodes: TopLevelNodeKind[] | null;
	MetaData: MetaData;
}

export interface Group {
	Type: GroupType;
	Id: string | null;
	Sections: Section[] | null;
	MetaData: MetaData;
}

export interface Section {
	Type: SectionType;
	Name: string;
	Args: Argument[] | null;
	MetaData: MetaData;
}

export interface Argument {
	Type: ArgumentType;
	IsInline: boolean;
	Arg: ArgumentDataKind;
	MetaData: MetaData;
}

export type ArgumentDataKind = Group |
  TextArgumentData | FormulationArgumentData |
  ArgumentTextArgumentData | null;

export interface TextArgumentData {
	Type: TextArgumentDataKind;
  Text: string;
  MetaData: MetaData;
}

export interface FormulationArgumentData {
	Type: FormulationArgumentDataKind;
	Text: string;
	MetaData: MetaData;
}

export interface ArgumentTextArgumentData {
	Type: ArgumentTextArgumentDataKind;
	Text: string;
	MetaData: MetaData;
}
