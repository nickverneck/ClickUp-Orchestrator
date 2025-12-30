<script lang="ts">
	interface Props {
		previewUrl: string;
		isMobileView: boolean;
		highlightMode: boolean;
		onUrlChange: (url: string) => void;
		onMobileToggle: () => void;
		onHighlightToggle: () => void;
		onRefresh: () => void;
	}

	let {
		previewUrl,
		isMobileView,
		highlightMode,
		onUrlChange,
		onMobileToggle,
		onHighlightToggle,
		onRefresh
	}: Props = $props();

	let urlInput = $state(previewUrl);
	let isEditing = $state(false);

	function handleUrlSubmit() {
		if (urlInput.trim() && urlInput !== previewUrl) {
			onUrlChange(urlInput.trim());
		}
		isEditing = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleUrlSubmit();
		} else if (event.key === 'Escape') {
			urlInput = previewUrl;
			isEditing = false;
		}
	}

	$effect(() => {
		urlInput = previewUrl;
	});
</script>

<div class="flex items-center gap-2 px-4 py-3 bg-white border-b border-gray-200">
	<!-- Refresh Button -->
	<button
		onclick={onRefresh}
		class="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
		title="Refresh preview"
	>
		<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
			/>
		</svg>
	</button>

	<!-- URL Bar -->
	<div class="flex-1 relative">
		{#if isEditing}
			<input
				type="text"
				bind:value={urlInput}
				onblur={handleUrlSubmit}
				onkeydown={handleKeydown}
				class="w-full px-3 py-1.5 text-sm bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent font-mono"
				autofocus
			/>
		{:else}
			<button
				onclick={() => (isEditing = true)}
				class="w-full px-3 py-1.5 text-sm text-left bg-gray-100 hover:bg-gray-200 rounded-lg font-mono text-gray-600 truncate transition-colors"
			>
				{previewUrl}
			</button>
		{/if}
	</div>

	<!-- Divider -->
	<div class="w-px h-6 bg-gray-200"></div>

	<!-- Mobile Toggle -->
	<button
		onclick={onMobileToggle}
		class="p-2 rounded-lg transition-colors {isMobileView
			? 'text-indigo-600 bg-indigo-50 hover:bg-indigo-100'
			: 'text-gray-500 hover:text-gray-700 hover:bg-gray-100'}"
		title={isMobileView ? 'Switch to desktop view' : 'Switch to mobile view'}
	>
		{#if isMobileView}
			<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z"
				/>
			</svg>
		{:else}
			<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
				/>
			</svg>
		{/if}
	</button>

	<!-- Highlight Toggle -->
	<button
		onclick={onHighlightToggle}
		class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg transition-colors text-sm font-medium {highlightMode
			? 'text-white bg-indigo-600 hover:bg-indigo-700'
			: 'text-gray-600 bg-gray-100 hover:bg-gray-200'}"
		title={highlightMode ? 'Disable highlight mode' : 'Enable highlight mode'}
	>
		<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5"
			/>
		</svg>
		<span>Highlight</span>
	</button>
</div>
