
export type TextBlockType = "TextBlockType";
export type RootType = "RootType";
export type GroupType = "GroupType";
export type SectionType = "SectionType";
export type ArgumentType = "ArgumentType";
export type TextArgumentDataType = "TextArgumentDataType";
export type FormulationArgumentDataType = "FormulationArgumentDataType";
export type ArgumentTextArgumentDataType = "ArgumentTextArgumentDataType";

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

export interface PathsResponse {
	Error: string;
	Paths: string[] | null;
}

export interface PageResponse {
	Error: string;
	Diagnostics: Diagnostic[] | null;
	Root: Root;
}

export interface MetaData {
  Start: Position;
}

export interface TextBlock {
	Type: TextBlockType;
	Text: string;
	MetaData: MetaData;
}

export type TopLevelNodeType = TextBlock | Group | null;

export interface Root {
	Type: RootType;
	Nodes: TopLevelNodeType[] | null;
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
	Arg: ArgumentDataType;
	MetaData: MetaData;
}

export type ArgumentDataType = Group |
  TextArgumentData | FormulationArgumentData |
  ArgumentTextArgumentData | null;

export interface TextArgumentData {
	Type: TextArgumentDataType;
  Text: string;
  MetaData: MetaData;
}

export interface FormulationArgumentData {
	Type: FormulationArgumentDataType;
	Text: string;
	MetaData: MetaData;
}

export interface ArgumentTextArgumentData {
	Type: ArgumentTextArgumentDataType;
	Text: string;
	MetaData: MetaData;
}
