// WebSocket connection manager for terminal streaming

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
	task_id: number;
	is_running: boolean;
}

export interface ErrorMessage {
	type: 'error';
	message: string;
}

export type WsMessage = OutputMessage | ConnectedMessage | ErrorMessage;

export class TerminalWebSocket {
	private ws: WebSocket | null = null;
	private taskId: number;
	private onMessage: (msg: WsMessage) => void;
	private onClose: () => void;
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;

	constructor(
		taskId: number,
		onMessage: (msg: WsMessage) => void,
		onClose: () => void
	) {
		this.taskId = taskId;
		this.onMessage = onMessage;
		this.onClose = onClose;
	}

	connect(): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			return;
		}

		this.ws = new WebSocket(`${WS_BASE}/ws/tasks/${this.taskId}/terminal`);

		this.ws.onopen = () => {
			console.log(`WebSocket connected for task ${this.taskId}`);
			this.reconnectAttempts = 0;
		};

		this.ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data) as WsMessage;
				this.onMessage(msg);
			} catch (e) {
				console.error('Failed to parse WebSocket message:', e);
			}
		};

		this.ws.onerror = (error) => {
			console.error('WebSocket error:', error);
		};

		this.ws.onclose = () => {
			console.log(`WebSocket closed for task ${this.taskId}`);
			this.onClose();

			// Auto-reconnect
			if (this.reconnectAttempts < this.maxReconnectAttempts) {
				this.reconnectAttempts++;
				setTimeout(() => this.connect(), 1000 * this.reconnectAttempts);
			}
		};
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
