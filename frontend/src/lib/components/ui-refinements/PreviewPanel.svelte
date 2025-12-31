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

	let iframeRef = $state<HTMLIFrameElement | null>(null);
	let iframeKey = $state(0);

	// Manual element selector state
	let showManualSelector = $state(false);
	let manualTagName = $state('');
	let manualClasses = $state('');
	let manualId = $state('');
	let manualDescription = $state('');

	function handleRefresh() {
		iframeKey++;
	}

	function handleOverlayClick() {
		showManualSelector = true;
	}

	function handleManualSelect() {
		if (!manualTagName.trim()) return;

		const metadata: ElementMetadata = {
			tagName: manualTagName.trim().toLowerCase(),
			id: manualId.trim() || undefined,
			classList: manualClasses.trim() ? manualClasses.trim().split(/\s+/) : [],
			attributes: {},
			textContent: manualDescription.trim() || undefined,
			xpath: '',
			cssSelector: buildCssSelector()
		};

		onElementSelect(metadata);
		resetManualForm();
		onHighlightToggle(); // Turn off highlight mode
	}

	function buildCssSelector(): string {
		let selector = manualTagName.trim().toLowerCase();
		if (manualId.trim()) {
			selector += `#${manualId.trim()}`;
		}
		if (manualClasses.trim()) {
			selector += '.' + manualClasses.trim().split(/\s+/).join('.');
		}
		return selector;
	}

	function resetManualForm() {
		showManualSelector = false;
		manualTagName = '';
		manualClasses = '';
		manualId = '';
		manualDescription = '';
	}

	function handleCancelManual() {
		resetManualForm();
	}
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
	<div class="flex-1 p-4 overflow-hidden relative">
		<div
			class="h-full bg-white rounded-lg shadow-lg overflow-hidden mx-auto transition-all duration-300 relative {isMobileView
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
					Click anywhere to describe an element
				</div>

				<!-- Overlay for capturing clicks -->
				<button
					onclick={handleOverlayClick}
					class="absolute inset-0 top-7 bg-indigo-600/10 hover:bg-indigo-600/20 cursor-crosshair transition-colors z-10 border-2 border-dashed border-indigo-400"
				>
					<span class="sr-only">Click to select element</span>
				</button>
			{/if}

			{#key iframeKey}
				<iframe
					bind:this={iframeRef}
					src={previewUrl}
					title="Preview"
					class="w-full border-0 {highlightMode ? 'h-[calc(100%-28px)]' : 'h-full'}"
					sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
				></iframe>
			{/key}
		</div>

		<!-- Manual Element Selector Modal -->
		{#if showManualSelector}
			<div class="absolute inset-0 bg-gray-900/50 flex items-center justify-center z-20 rounded-lg">
				<div class="bg-white rounded-lg shadow-xl max-w-sm w-full mx-4 overflow-hidden">
					<div class="bg-indigo-600 px-4 py-3">
						<h3 class="text-white font-medium">Describe the Element</h3>
						<p class="text-indigo-200 text-xs mt-0.5">
							Use browser DevTools (right-click → Inspect) to find element details
						</p>
					</div>
					<div class="p-4 space-y-3">
						<div>
							<label for="tagName" class="block text-xs font-medium text-gray-700 mb-1">
								Tag Name <span class="text-red-500">*</span>
							</label>
							<input
								id="tagName"
								type="text"
								bind:value={manualTagName}
								placeholder="button, div, input, etc."
								class="w-full px-3 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
							/>
						</div>
						<div>
							<label for="elementId" class="block text-xs font-medium text-gray-700 mb-1">
								ID (optional)
							</label>
							<input
								id="elementId"
								type="text"
								bind:value={manualId}
								placeholder="submit-btn"
								class="w-full px-3 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono"
							/>
						</div>
						<div>
							<label for="classes" class="block text-xs font-medium text-gray-700 mb-1">
								Classes (space-separated, optional)
							</label>
							<input
								id="classes"
								type="text"
								bind:value={manualClasses}
								placeholder="btn btn-primary"
								class="w-full px-3 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono"
							/>
						</div>
						<div>
							<label for="description" class="block text-xs font-medium text-gray-700 mb-1">
								Description (optional)
							</label>
							<input
								id="description"
								type="text"
								bind:value={manualDescription}
								placeholder="The blue submit button at the bottom"
								class="w-full px-3 py-1.5 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
							/>
						</div>

						{#if manualTagName.trim()}
							<div class="text-xs text-gray-500 bg-gray-50 p-2 rounded font-mono">
								Selector: {buildCssSelector()}
							</div>
						{/if}
					</div>
					<div class="px-4 py-3 bg-gray-50 flex gap-2">
						<button
							onclick={handleCancelManual}
							class="flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded-md hover:bg-gray-100"
						>
							Cancel
						</button>
						<button
							onclick={handleManualSelect}
							disabled={!manualTagName.trim()}
							class="flex-1 px-3 py-1.5 text-sm bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:opacity-50"
						>
							Select Element
						</button>
					</div>
				</div>
			</div>
		{/if}
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
