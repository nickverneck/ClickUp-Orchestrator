<script lang="ts">
	import type { ElementMetadata } from '$lib/types/ui-refinements';
	import PreviewControls from './PreviewControls.svelte';

	interface Props {
		previewUrl: string;
		isMobileView: boolean;
		highlightMode: boolean;
		onUrlChange: (url: string) => void;
		onMobileToggle: () => void;
		onHighlightToggle: () => void;
		onElementSelect: (element: ElementMetadata) => void;
	}

	let {
		previewUrl,
		isMobileView,
		highlightMode,
		onUrlChange,
		onMobileToggle,
		onHighlightToggle,
		onElementSelect
	}: Props = $props();

	let iframeRef: HTMLIFrameElement;
	let iframeKey = $state(0);

	function handleRefresh() {
		iframeKey++;
	}

	function handleMessage(event: MessageEvent) {
		if (event.data?.type === 'element-selected') {
			onElementSelect(event.data.metadata);
		}
	}

	$effect(() => {
		window.addEventListener('message', handleMessage);
		return () => window.removeEventListener('message', handleMessage);
	});

	// Build the proxied URL when highlight mode is on
	let displayUrl = $derived(
		highlightMode ? `/api/ui-refinements/proxy?url=${encodeURIComponent(previewUrl)}` : previewUrl
	);
</script>

<div class="flex flex-col h-full">
	<!-- Controls -->
	<PreviewControls
		{previewUrl}
		{isMobileView}
		{highlightMode}
		{onUrlChange}
		{onMobileToggle}
		{onHighlightToggle}
		onRefresh={handleRefresh}
	/>

	<!-- Preview Container -->
	<div class="flex-1 p-4 overflow-hidden">
		<div
			class="h-full bg-white rounded-lg shadow-lg overflow-hidden mx-auto transition-all duration-300 {isMobileView
				? 'max-w-[375px]'
				: 'w-full'}"
		>
			{#if highlightMode}
				<div class="bg-indigo-600 text-white text-xs text-center py-1.5 px-3">
					<span class="animate-pulse mr-2">
						<svg class="w-3 h-3 inline" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M15 15l-2 5L9 9l11 4-5 2z"
							/>
						</svg>
					</span>
					Highlight Mode: Click on any element to select it
				</div>
			{/if}

			{#key iframeKey}
				<iframe
					bind:this={iframeRef}
					src={displayUrl}
					title="Preview"
					class="w-full border-0 {highlightMode ? 'h-[calc(100%-28px)]' : 'h-full'}"
					sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
				></iframe>
			{/key}
		</div>
	</div>

	<!-- Mobile Indicator -->
	{#if isMobileView}
		<div class="text-center pb-2">
			<span class="text-xs text-gray-500">
				Mobile Preview (375px)
			</span>
		</div>
	{/if}
</div>
