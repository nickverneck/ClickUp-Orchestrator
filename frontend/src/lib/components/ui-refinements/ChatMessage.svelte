<script lang="ts">
	import type { ChatMessage } from '$lib/types/ui-refinements';

	interface Props {
		message: ChatMessage;
	}

	let { message }: Props = $props();

	let isUser = $derived(message.role === 'user');
</script>

<div class="flex {isUser ? 'justify-end' : 'justify-start'}">
	<div
		class="max-w-[80%] rounded-xl px-4 py-2.5 {isUser
			? 'bg-indigo-600 text-white'
			: 'bg-gray-100 text-gray-900'}"
	>
		<!-- Element Context Badge -->
		{#if message.elementContext}
			<div
				class="flex items-center gap-1.5 mb-2 text-xs {isUser
					? 'text-indigo-200'
					: 'text-gray-500'}"
			>
				<svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5"
					/>
				</svg>
				<span class="font-mono">
					&lt;{message.elementContext.tagName}{message.elementContext.classList.length > 0
						? ` .${message.elementContext.classList[0]}`
						: ''}&gt;
				</span>
			</div>
		{/if}

		<!-- Message Content -->
		<p class="text-sm whitespace-pre-wrap">{message.content}</p>

		<!-- Status Indicator -->
		<div class="flex items-center justify-end gap-1.5 mt-1.5">
			{#if message.status === 'processing'}
				<svg class="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24">
					<circle
						class="opacity-25"
						cx="12"
						cy="12"
						r="10"
						stroke="currentColor"
						stroke-width="4"
					></circle>
					<path
						class="opacity-75"
						fill="currentColor"
						d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
					></path>
				</svg>
			{:else if message.status === 'completed'}
				<svg class="w-3 h-3 {isUser ? 'text-indigo-200' : 'text-green-500'}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
				</svg>
			{:else if message.status === 'error'}
				<svg class="w-3 h-3 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
				</svg>
			{/if}
			<span class="text-[10px] {isUser ? 'text-indigo-200' : 'text-gray-400'}">
				{new Date(message.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
			</span>
		</div>
	</div>
</div>
