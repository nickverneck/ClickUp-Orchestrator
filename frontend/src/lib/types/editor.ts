export interface FileNode {
	name: string;
	path: string;
	isDirectory: boolean;
	children?: FileNode[];
}

export interface EditorTab {
	id: string;
	path: string;
	name: string;
	content: string;
	language: string;
	isDirty: boolean;
}

export interface FileContent {
	content: string;
	language: string;
	encoding: string;
}
