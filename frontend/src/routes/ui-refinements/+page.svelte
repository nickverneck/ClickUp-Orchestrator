<script lang="ts">
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import BranchProtectionOverlay from '$lib/components/ui-refinements/BranchProtectionOverlay.svelte';
	import ChatPanel from '$lib/components/ui-refinements/ChatPanel.svelte';
	import PreviewPanel from '$lib/components/ui-refinements/PreviewPanel.svelte';
	import type { ChatMessage, QueuedMessage, ElementMetadata } from '$lib/types/ui-refinements';

	let sidebarCollapsed = $state(false);

	// Branch protection state
	let branchCreated = $state(false);
	let currentBranch = $state<string | null>(null);

	// Chat state
	let messages = $state<ChatMessage[]>([]);
	let messageQueue = $state<QueuedMessage[]>([]);
	let isProcessing = $state(false);
	let selectedAgent = $state<'claude' | 'codex' | 'gemini'>('claude');

	// Preview state
	let previewUrl = $state('http://localhost:5173');
	let isMobileView = $state(false);
	let highlightMode = $state(false);
	let selectedElement = $state<ElementMetadata | null>(null);

	// Project state
	let selectedProject = $state<string | null>(null);

	onMount(() => {
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}

		const savedPreviewUrl = localStorage.getItem('uiRefinementsPreviewUrl');
		if (savedPreviewUrl) {
			previewUrl = savedPreviewUrl;
		}
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

	async function processMessage(message: ChatMessage) {
		// TODO: Implement WebSocket communication with backend
		// For now, simulate a response
		setTimeout(() => {
			messages = [
				...messages,
				{
					id: crypto.randomUUID(),
					role: 'assistant',
					content: `Processing your request with ${selectedAgent}...`,
					timestamp: Date.now(),
					status: 'completed'
				}
			];
			isProcessing = false;
			processNextInQueue();
		}, 2000);
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
			<div class="w-1/5 min-w-[280px] border-r border-gray-200 flex flex-col bg-white {!branchCreated ? 'blur-sm pointer-events-none' : ''}">
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
