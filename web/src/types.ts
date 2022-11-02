
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
	Paths: string[];
}

export interface PageResponse {
	Error: string;
	Diagnostics: Diagnostic[];
	Root: Root;
	Html: string;
}

export interface MetaData {
  Start: Position;
}

export interface TextBlock {
	Text: string;
	MetaData: MetaData;
}

export type TopLevelNodeType = TextBlock | Group;

export interface Root {
	Nodes: TopLevelNodeType[];
	MetaData: MetaData;
}

export interface Group {
	Id?: string;
	Sections: Section[];
	MetaData: MetaData;
}

export interface Section {
	Name: string;
	Args: Argument[];
	MetaData: MetaData;
}

export interface Argument {
	IsInline: boolean;
	Arg: ArgumentDataType;
	MetaData: MetaData;
}

export type ArgumentDataType = Group |
  TextArgumentData | FormulationArgumentData |
  ArgumentTextArgumentData;

export interface TextArgumentData {
  Text: string;
  MetaData: MetaData;
}

export interface FormulationArgumentData {
	Text: string;
	MetaData: MetaData;
}

export interface ArgumentTextArgumentData {
	Text: string;
	MetaData: MetaData;
}
