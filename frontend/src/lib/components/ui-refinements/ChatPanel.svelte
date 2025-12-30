<script lang="ts">
	import type { ChatMessage, QueuedMessage, ElementMetadata, AgentType } from '$lib/types/ui-refinements';
	import ChatMessageComponent from './ChatMessage.svelte';
	import AgentSelector from './AgentSelector.svelte';
	import MessageInput from './MessageInput.svelte';
	import ElementMetadataCard from './ElementMetadataCard.svelte';

	interface Props {
		messages: ChatMessage[];
		messageQueue: QueuedMessage[];
		isProcessing: boolean;
		selectedAgent: AgentType;
		selectedElement: ElementMetadata | null;
		onSendMessage: (content: string) => void;
		onAgentChange: (agent: AgentType) => void;
		onCancelQueue: (messageId: string) => void;
		onClearElement: () => void;
	}

	let {
		messages,
		messageQueue,
		isProcessing,
		selectedAgent,
		selectedElement,
		onSendMessage,
		onAgentChange,
		onCancelQueue,
		onClearElement
	}: Props = $props();

	let messagesContainer: HTMLDivElement;

	$effect(() => {
		// Auto-scroll to bottom when new messages arrive
		if (messagesContainer && messages.length > 0) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	});
</script>

<div class="flex flex-col h-full">
	<!-- Header -->
	<div class="px-4 py-3 border-b border-gray-200 bg-gray-50">
		<div class="flex items-center justify-between">
			<h3 class="font-medium text-gray-900">Chat</h3>
			<AgentSelector selected={selectedAgent} onChange={onAgentChange} />
		</div>
	</div>

	<!-- Messages -->
	<div bind:this={messagesContainer} class="flex-1 overflow-y-auto p-4 space-y-4">
		{#if messages.length === 0}
			<div class="flex flex-col items-center justify-center h-full text-center text-gray-500">
				<svg class="w-12 h-12 mb-3 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
					/>
				</svg>
				<p class="font-medium">No messages yet</p>
				<p class="text-sm mt-1">
					Use the highlight tool to select elements, then describe your changes.
				</p>
			</div>
		{:else}
			{#each messages as message (message.id)}
				<ChatMessageComponent {message} />
			{/each}
		{/if}
	</div>

	<!-- Queue Indicator -->
	{#if messageQueue.length > 0}
		<div class="px-4 py-2 bg-amber-50 border-t border-amber-200">
			<div class="flex items-center justify-between text-sm">
				<span class="text-amber-700">
					<span class="font-medium">{messageQueue.length}</span> message{messageQueue.length > 1 ? 's' : ''} queued
				</span>
				<button
					onclick={() => onCancelQueue(messageQueue[0].id)}
					class="text-amber-600 hover:text-amber-800 font-medium"
				>
					Cancel Next
				</button>
			</div>
		</div>
	{/if}

	<!-- Selected Element Context -->
	{#if selectedElement}
		<div class="border-t border-gray-200">
			<ElementMetadataCard element={selectedElement} onClear={onClearElement} />
		</div>
	{/if}

	<!-- Input -->
	<div class="border-t border-gray-200 p-4">
		<MessageInput
			{isProcessing}
			hasElementContext={!!selectedElement}
			{onSendMessage}
		/>
	</div>
</div>
