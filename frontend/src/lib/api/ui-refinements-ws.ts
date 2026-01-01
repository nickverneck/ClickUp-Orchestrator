// WebSocket connection manager for UI Refinements chat streaming

// Use the current hostname to support LAN access
const WS_HOST = typeof window !== 'undefined' ? window.location.hostname : 'localhost';
const WS_BASE = `ws://${WS_HOST}:5150`;

export interface OutputMessage {
	type: 'output';
	line: string;
	is_stderr: boolean;
}

export interface ConnectedMessage {
	type: 'connected';
	session_id: string;
	is_running: boolean;
}

export interface ErrorMessage {
	type: 'error';
	message: string;
}

export interface SpawnedMessage {
	type: 'spawned';
	pid: number;
}

export type SessionWsMessage = OutputMessage | ConnectedMessage | ErrorMessage | SpawnedMessage;

export class UIRefinementsWebSocket {
	private ws: WebSocket | null = null;
	private sessionId: string;
	private onMessage: (msg: SessionWsMessage) => void;
	private onClose: () => void;
	private onConnect: (isRunning: boolean) => void;
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;

	constructor(
		sessionId: string,
		onMessage: (msg: SessionWsMessage) => void,
		onClose: () => void,
		onConnect: (isRunning: boolean) => void
	) {
		this.sessionId = sessionId;
		this.onMessage = onMessage;
		this.onClose = onClose;
		this.onConnect = onConnect;
	}

	connect(): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			return;
		}

		this.ws = new WebSocket(`${WS_BASE}/ws/ui-refinements/${this.sessionId}`);

		this.ws.onopen = () => {
			console.log(`WebSocket connected for session ${this.sessionId}`);
			this.reconnectAttempts = 0;
		};

		this.ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data) as SessionWsMessage;
				if (msg.type === 'connected') {
					this.onConnect((msg as ConnectedMessage).is_running);
				} else {
					this.onMessage(msg);
				}
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

	spawnAgent(prompt: string, agent: string, worktreePath: string): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(
				JSON.stringify({
					type: 'spawn',
					prompt,
					agent,
					worktree_path: worktreePath
				})
			);
		}
	}

	sendInput(data: string): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify({ type: 'input', data }));
		}
	}

	sendKill(): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify({ type: 'kill' }));
		}
	}

	disconnect(): void {
		this.maxReconnectAttempts = 0; // Prevent reconnection
		this.ws?.close();
		this.ws = null;
	}

	isConnected(): boolean {
		return this.ws?.readyState === WebSocket.OPEN;
	}
}
