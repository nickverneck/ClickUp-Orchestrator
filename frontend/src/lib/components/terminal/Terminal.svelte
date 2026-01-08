<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Terminal } from '@xterm/xterm';
	import { FitAddon } from '@xterm/addon-fit';
	import { TerminalWebSocket, type WsMessage } from '$lib/api/websocket';
	import '@xterm/xterm/css/xterm.css';

	interface Props {
		taskId: number;
		onKill?: () => void;
	}

	let { taskId, onKill }: Props = $props();

	let terminalRef: HTMLDivElement;
	let terminal: Terminal;
	let fitAddon: FitAddon;
	let ws: TerminalWebSocket;
	let connected = $state(false);
	let isRunning = $state(false);

	onMount(() => {
		// Initialize terminal
		terminal = new Terminal({
			cursorBlink: true,
			theme: {
				background: '#1e1e1e',
				foreground: '#d4d4d4',
				cursor: '#d4d4d4',
				selectionBackground: '#264f78',
				black: '#1e1e1e',
				red: '#f44747',
				green: '#6a9955',
				yellow: '#dcdcaa',
				blue: '#569cd6',
				magenta: '#c586c0',
				cyan: '#4ec9b0',
				white: '#d4d4d4'
			},
			fontFamily: 'Menlo, Monaco, "Courier New", monospace',
			fontSize: 13,
			lineHeight: 1.2
		});

		fitAddon = new FitAddon();
		terminal.loadAddon(fitAddon);
		terminal.open(terminalRef);
		fitAddon.fit();

		// Handle terminal input
		terminal.onData((data) => {
			ws?.sendInput(data);
		});

		// Handle resize
		const resizeObserver = new ResizeObserver(() => {
			fitAddon.fit();
		});
		resizeObserver.observe(terminalRef);

		// Connect WebSocket
		ws = new TerminalWebSocket(
			taskId,
			handleMessage,
			() => {
				connected = false;
			}
		);
		ws.connect();

		return () => {
			resizeObserver.disconnect();
			ws?.disconnect();
			terminal?.dispose();
		};
	});

	onDestroy(() => {
		ws?.disconnect();
		terminal?.dispose();
	});

	function handleMessage(msg: WsMessage) {
		switch (msg.type) {
			case 'connected':
				connected = true;
				isRunning = msg.is_running;
				terminal.writeln('\x1b[32m[Connected to terminal]\x1b[0m');
				if (!msg.is_running) {
					terminal.writeln('\x1b[33m[Process is not running]\x1b[0m');
				}
				break;
			case 'output':
				if (msg.line.includes('Process exited with code')) {
					isRunning = false;
				} else if (!isRunning) {
					isRunning = true;
				}
				if (msg.is_stderr) {
					terminal.write(`\x1b[31m${msg.line}\x1b[0m\r\n`);
				} else {
					terminal.write(msg.line + '\r\n');
				}
				break;
			case 'error':
				terminal.writeln(`\x1b[31m[Error: ${msg.message}]\x1b[0m`);
				break;
		}
	}

	function handleKill() {
		ws?.sendKill();
		onKill?.();
	}
</script>

<div class="flex flex-col h-full">
	<!-- Terminal header -->
	<div class="flex items-center justify-between bg-gray-800 px-4 py-2">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-1.5">
				<div class="h-3 w-3 rounded-full bg-red-500"></div>
				<div class="h-3 w-3 rounded-full bg-yellow-500"></div>
				<div class="h-3 w-3 rounded-full bg-green-500"></div>
			</div>
			<span class="text-sm text-gray-400">Task #{taskId}</span>
			{#if connected}
				<span class="inline-flex items-center gap-1 text-xs text-green-400">
					<span class="h-1.5 w-1.5 rounded-full bg-green-400"></span>
					Connected
				</span>
			{:else}
				<span class="inline-flex items-center gap-1 text-xs text-red-400">
					<span class="h-1.5 w-1.5 rounded-full bg-red-400"></span>
					Disconnected
				</span>
			{/if}
		</div>

		{#if isRunning}
			<button
				onclick={handleKill}
				class="inline-flex items-center gap-1 rounded bg-red-600 px-3 py-1 text-sm font-medium text-white hover:bg-red-700"
			>
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
				</svg>
				Kill Process
			</button>
		{/if}
	</div>

	<!-- Terminal body -->
	<div bind:this={terminalRef} class="flex-1 bg-[#1e1e1e]"></div>
</div>
