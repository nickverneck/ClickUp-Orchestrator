import { get, post, del } from './client';
import type { AgentType, ElementMetadata } from '$lib/types/ui-refinements';

const API_BASE = 'http://localhost:5150';

export interface Session {
	session_id: string;
	branch_name: string;
}

export interface QueueStatus {
	pending_messages: number;
	current_task?: string;
}

export async function createSession(branchName: string): Promise<Session> {
	return post<Session>('/ui-refinements/session', { branch_name: branchName });
}

export async function sendMessage(
	sessionId: string,
	message: string,
	agent: AgentType,
	elementContext?: ElementMetadata
): Promise<{ success: boolean; queued: boolean; queue_position?: number }> {
	return post('/ui-refinements/chat', {
		session_id: sessionId,
		message,
		agent,
		element_context: elementContext
	});
}

export async function getQueueStatus(sessionId: string): Promise<QueueStatus> {
	return get<QueueStatus>(`/ui-refinements/queue/${sessionId}`);
}

export async function cancelQueuedMessage(
	sessionId: string,
	messageId: string
): Promise<{ success: boolean }> {
	return del<{ success: boolean }>(`/ui-refinements/queue/${sessionId}/${messageId}`);
}

export function getProxyUrl(targetUrl: string): string {
	return `${API_BASE}/api/ui-refinements/proxy?url=${encodeURIComponent(targetUrl)}`;
}

// WebSocket client for UI Refinements
export class UIRefinementsWebSocket {
	private ws: WebSocket | null = null;
	private sessionId: string;
	private onMessage: (msg: UIRefinementsWsMessage) => void;
	private onClose: () => void;
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;

	constructor(
		sessionId: string,
		onMessage: (msg: UIRefinementsWsMessage) => void,
		onClose: () => void
	) {
		this.sessionId = sessionId;
		this.onMessage = onMessage;
		this.onClose = onClose;
	}

	connect(): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			return;
		}

		this.ws = new WebSocket(`ws://localhost:5150/ws/ui-refinements/${this.sessionId}`);

		this.ws.onopen = () => {
			console.log(`WebSocket connected for session ${this.sessionId}`);
			this.reconnectAttempts = 0;
		};

		this.ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data) as UIRefinementsWsMessage;
				this.onMessage(msg);
			} catch (e) {
				console.error('Failed to parse WebSocket message:', e);
			}
		};

		this.ws.onerror = (error) => {
			console.error('WebSocket error:', error);
		};

		this.ws.onclose = () => {
			console.log(`WebSocket closed for session ${this.sessionId}`);
			this.onClose();

			// Auto-reconnect
			if (this.reconnectAttempts < this.maxReconnectAttempts) {
				this.reconnectAttempts++;
				setTimeout(() => this.connect(), 1000 * this.reconnectAttempts);
			}
		};
	}

	send(message: UIRefinementsWsOutMessage): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(message));
		}
	}

	disconnect(): void {
		this.maxReconnectAttempts = 0;
		this.ws?.close();
		this.ws = null;
	}

	isConnected(): boolean {
		return this.ws?.readyState === WebSocket.OPEN;
	}
}

export interface UIRefinementsWsMessage {
	type: 'response' | 'task_complete' | 'queue_update' | 'error';
	content?: string;
	is_complete?: boolean;
	success?: boolean;
	message?: string;
	pending?: number;
	current?: string;
}

export interface UIRefinementsWsOutMessage {
	type: 'chat' | 'cancel';
	message?: string;
	agent?: AgentType;
	element_context?: ElementMetadata;
	message_id?: string;
}
