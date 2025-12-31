<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import BranchProtectionOverlay from '$lib/components/ui-refinements/BranchProtectionOverlay.svelte';
	import ChatPanel from '$lib/components/ui-refinements/ChatPanel.svelte';
	import PreviewPanel from '$lib/components/ui-refinements/PreviewPanel.svelte';
	import type { ChatMessage, QueuedMessage, ElementMetadata } from '$lib/types/ui-refinements';
	import { UIRefinementsWebSocket, type SessionWsMessage } from '$lib/api/ui-refinements-ws';
	import { getSettings } from '$lib/api/settings';

	let sidebarCollapsed = $state(false);

	// Branch protection state
	let branchCreated = $state(false);
	let currentBranch = $state<string | null>(null);

	// Session state
	let sessionId = $state<string | null>(null);
	let wsClient = $state<UIRefinementsWebSocket | null>(null);
	let targetRepoPath = $state<string>('');

	// Chat state
	let messages = $state<ChatMessage[]>([]);
	let messageQueue = $state<QueuedMessage[]>([]);
	let isProcessing = $state(false);
	let selectedAgent = $state<'claude' | 'codex' | 'gemini'>('claude');
	let currentAssistantMessageId = $state<string | null>(null);

	// Preview state
	let previewUrl = $state('http://localhost:5173');
	let isMobileView = $state(false);
	let highlightMode = $state(false);
	let selectedElement = $state<ElementMetadata | null>(null);

	// Project state
	let selectedProject = $state<string | null>(null);

	onMount(async () => {
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}

		const savedPreviewUrl = localStorage.getItem('uiRefinementsPreviewUrl');
		if (savedPreviewUrl) {
			previewUrl = savedPreviewUrl;
		}

		// Load target repo path from settings
		try {
			const settings = await getSettings();
			if (settings.target_repo_path) {
				targetRepoPath = settings.target_repo_path;
			}
		} catch (e) {
			console.error('Failed to load settings:', e);
		}
	});

	onDestroy(() => {
		wsClient?.disconnect();
	});

	$effect(() => {
		localStorage.setItem('sidebarCollapsed', String(sidebarCollapsed));
	});

	$effect(() => {
		localStorage.setItem('uiRefinementsPreviewUrl', previewUrl);
	});

	function handleBranchCreate(branchName: string) {
		currentBranch = branchName;
		branchCreated = true;

		// Create a new session ID and connect WebSocket
		sessionId = crypto.randomUUID();
		connectWebSocket();
	}

	function connectWebSocket() {
		if (!sessionId) return;

		wsClient = new UIRefinementsWebSocket(
			sessionId,
			handleWsMessage,
			handleWsClose,
			handleWsConnect
		);
		wsClient.connect();
	}

	function handleWsMessage(msg: SessionWsMessage) {
		if (msg.type === 'output') {
			// Append output to current assistant message
			if (currentAssistantMessageId) {
				messages = messages.map((m) =>
					m.id === currentAssistantMessageId
						? { ...m, content: m.content + msg.line + '\n' }
						: m
				);
			}

			// Check if process exited
			if (msg.line.includes('[Process exited with code')) {
				// Mark message as completed and process next
				if (currentAssistantMessageId) {
					messages = messages.map((m) =>
						m.id === currentAssistantMessageId ? { ...m, status: 'completed' as const } : m
					);
				}
				currentAssistantMessageId = null;
				isProcessing = false;
				processNextInQueue();
			}
		} else if (msg.type === 'error') {
			// Show error message
			if (currentAssistantMessageId) {
				messages = messages.map((m) =>
					m.id === currentAssistantMessageId
						? { ...m, content: m.content + `\nError: ${msg.message}`, status: 'error' as const }
						: m
				);
			}
			currentAssistantMessageId = null;
			isProcessing = false;
			processNextInQueue();
		}
	}

	function handleWsClose() {
		console.log('WebSocket closed');
	}

	function handleWsConnect(isRunning: boolean) {
		console.log('WebSocket connected, isRunning:', isRunning);
	}

	function handleElementSelect(element: ElementMetadata) {
		selectedElement = element;
		highlightMode = false;
	}

	function handleSendMessage(content: string) {
		const messageId = crypto.randomUUID();
		const newMessage: ChatMessage = {
			id: messageId,
			role: 'user',
			content,
			timestamp: Date.now(),
			elementContext: selectedElement ?? undefined,
			status: 'pending'
		};

		if (isProcessing) {
			// Add to queue
			messageQueue = [
				...messageQueue,
				{
					id: messageId,
					content,
					elementContext: selectedElement ?? undefined,
					queuePosition: messageQueue.length + 1
				}
			];
		} else {
			// Process immediately
			isProcessing = true;
			messages = [...messages, { ...newMessage, status: 'processing' }];
			processMessage(newMessage);
		}

		// Clear selected element after sending
		selectedElement = null;
	}

	function buildPrompt(message: ChatMessage): string {
		let prompt = message.content;

		// Add element context if present
		if (message.elementContext) {
			const ctx = message.elementContext;
			prompt += `\n\n[Element Context]\nTag: ${ctx.tagName}`;
			if (ctx.id) prompt += `\nID: ${ctx.id}`;
			if (ctx.classList.length > 0) prompt += `\nClasses: ${ctx.classList.join(' ')}`;
			if (ctx.cssSelector) prompt += `\nCSS Selector: ${ctx.cssSelector}`;
			if (ctx.textContent) prompt += `\nText Content: ${ctx.textContent}`;
		}

		return prompt;
	}

	async function processMessage(message: ChatMessage) {
		if (!wsClient || !targetRepoPath) {
			// Fallback if not connected
			messages = [
				...messages,
				{
					id: crypto.randomUUID(),
					role: 'assistant',
					content: 'Error: Not connected to backend or no target repository configured.',
					timestamp: Date.now(),
					status: 'error'
				}
			];
			isProcessing = false;
			processNextInQueue();
			return;
		}

		// Create assistant message placeholder
		const assistantId = crypto.randomUUID();
		currentAssistantMessageId = assistantId;
		messages = [
			...messages,
			{
				id: assistantId,
				role: 'assistant',
				content: '',
				timestamp: Date.now(),
				status: 'processing'
			}
		];

		// Build the prompt with element context
		const prompt = buildPrompt(message);

		// Spawn the agent via WebSocket
		wsClient.spawnAgent(prompt, selectedAgent, targetRepoPath);
	}

	function processNextInQueue() {
		if (messageQueue.length > 0) {
			const next = messageQueue[0];
			messageQueue = messageQueue.slice(1);
			const newMessage: ChatMessage = {
				id: next.id,
				role: 'user',
				content: next.content,
				timestamp: Date.now(),
				elementContext: next.elementContext,
				status: 'processing'
			};
			isProcessing = true;
			messages = [...messages, newMessage];
			processMessage(newMessage);
		}
	}

	function handleCancelQueue(messageId: string) {
		messageQueue = messageQueue.filter((m) => m.id !== messageId);
	}
</script>

<svelte:head>
	<title>UI Modifier - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex h-screen bg-gray-50">
	<Sidebar bind:collapsed={sidebarCollapsed} />

	<main class="flex-1 flex flex-col overflow-hidden">
		<!-- Header -->
		<div class="border-b border-gray-200 bg-white px-6 py-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<h1 class="text-xl font-bold text-gray-900">UI Modifier</h1>
					<span
						class="text-xs font-medium bg-amber-100 text-amber-700 px-2 py-1 rounded-full"
					>
						EXPERIMENTAL
					</span>
				</div>
				{#if currentBranch}
					<div class="flex items-center gap-2 text-sm">
						<svg class="w-4 h-4 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
						</svg>
						<span class="text-gray-600">Branch:</span>
						<span class="font-mono text-indigo-600">{currentBranch}</span>
					</div>
				{/if}
			</div>
		</div>

		<!-- Main Content -->
		<div class="flex-1 flex overflow-hidden relative">
			{#if !branchCreated}
				<BranchProtectionOverlay onBranchCreate={handleBranchCreate} />
			{/if}

			<!-- Chat Panel - Left Side -->
			<div class="w-[30%] min-w-[320px] border-r border-gray-200 flex flex-col bg-white {!branchCreated ? 'blur-sm pointer-events-none' : ''}">
				<ChatPanel
					{messages}
					{messageQueue}
					{isProcessing}
					{selectedAgent}
					{selectedElement}
					onSendMessage={handleSendMessage}
					onAgentChange={(agent) => (selectedAgent = agent)}
					onCancelQueue={handleCancelQueue}
					onClearElement={() => (selectedElement = null)}
				/>
			</div>

			<!-- Preview Panel - Right Side -->
			<div class="flex-1 flex flex-col bg-gray-100 {!branchCreated ? 'blur-sm pointer-events-none' : ''}">
				<PreviewPanel
					{previewUrl}
					{isMobileView}
					{highlightMode}
					onUrlChange={(url) => (previewUrl = url)}
					onMobileToggle={() => (isMobileView = !isMobileView)}
					onHighlightToggle={() => (highlightMode = !highlightMode)}
					onElementSelect={handleElementSelect}
				/>
			</div>
		</div>
	</main>
</div>
