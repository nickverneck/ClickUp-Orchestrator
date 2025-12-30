<script lang="ts">
	interface Props {
		isProcessing: boolean;
		hasElementContext: boolean;
		onSendMessage: (content: string) => void;
	}

	let { isProcessing, hasElementContext, onSendMessage }: Props = $props();

	let message = $state('');
	let textarea: HTMLTextAreaElement;

	function handleSubmit() {
		if (!message.trim()) return;
		onSendMessage(message.trim());
		message = '';
		if (textarea) {
			textarea.style.height = 'auto';
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			handleSubmit();
		}
	}

	function handleInput() {
		if (textarea) {
			textarea.style.height = 'auto';
			textarea.style.height = Math.min(textarea.scrollHeight, 150) + 'px';
		}
	}
</script>

<div class="flex gap-3">
	<div class="flex-1 relative">
		<textarea
			bind:this={textarea}
			bind:value={message}
			onkeydown={handleKeydown}
			oninput={handleInput}
			placeholder={hasElementContext
				? 'Describe what to change for the selected element...'
				: 'Describe your UI changes...'}
			rows="1"
			class="w-full px-4 py-2.5 pr-12 border border-gray-300 rounded-xl resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent placeholder-gray-400"
		></textarea>

		{#if hasElementContext}
			<div class="absolute right-12 top-1/2 -translate-y-1/2">
				<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-700">
					<svg class="w-3 h-3 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M15 15l-2 5L9 9l11 4-5 2z"
						/>
					</svg>
					Element
				</span>
			</div>
		{/if}
	</div>

	<button
		onclick={handleSubmit}
		disabled={!message.trim()}
		class="flex-shrink-0 w-10 h-10 flex items-center justify-center bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
	>
		{#if isProcessing}
			<svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
				></circle>
				<path
					class="opacity-75"
					fill="currentColor"
					d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
				></path>
			</svg>
		{:else}
			<svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
				/>
			</svg>
		{/if}
	</button>
</div>
