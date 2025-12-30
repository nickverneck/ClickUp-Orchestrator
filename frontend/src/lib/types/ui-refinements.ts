export type AgentType = 'claude' | 'codex' | 'gemini';

export interface ChatMessage {
	id: string;
	role: 'user' | 'assistant';
	content: string;
	timestamp: number;
	elementContext?: ElementMetadata;
	status: 'pending' | 'processing' | 'completed' | 'error';
}

export interface QueuedMessage {
	id: string;
	content: string;
	elementContext?: ElementMetadata;
	queuePosition: number;
}

export interface ElementMetadata {
	tagName: string;
	id?: string;
	classList: string[];
	attributes: Record<string, string>;
	textContent?: string;
	xpath: string;
	cssSelector: string;
	boundingRect?: {
		x: number;
		y: number;
		width: number;
		height: number;
	};
}

export interface UIRefinementSession {
	id: string;
	branchName: string;
	createdAt: number;
	projectPath: string;
}
