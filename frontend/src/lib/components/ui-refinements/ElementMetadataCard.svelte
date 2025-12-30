<script lang="ts">
	import type { ElementMetadata } from '$lib/types/ui-refinements';

	interface Props {
		element: ElementMetadata;
		onClear: () => void;
	}

	let { element, onClear }: Props = $props();
</script>

<div class="px-4 py-3 bg-indigo-50">
	<div class="flex items-start justify-between gap-3">
		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2 mb-1">
				<svg class="w-4 h-4 text-indigo-600 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15 15l-2 5L9 9l11 4-5 2z"
					/>
				</svg>
				<span class="text-sm font-medium text-indigo-900">Selected Element</span>
			</div>

			<div class="space-y-1 text-xs">
				<!-- Tag Name -->
				<div class="flex items-center gap-2">
					<span class="text-gray-500 w-12">Tag:</span>
					<code class="px-1.5 py-0.5 bg-indigo-100 text-indigo-800 rounded font-mono">
						&lt;{element.tagName}&gt;
					</code>
				</div>

				<!-- ID -->
				{#if element.id}
					<div class="flex items-center gap-2">
						<span class="text-gray-500 w-12">ID:</span>
						<code class="px-1.5 py-0.5 bg-gray-100 text-gray-800 rounded font-mono">
							#{element.id}
						</code>
					</div>
				{/if}

				<!-- Classes -->
				{#if element.classList.length > 0}
					<div class="flex items-start gap-2">
						<span class="text-gray-500 w-12 pt-0.5">Classes:</span>
						<div class="flex flex-wrap gap-1">
							{#each element.classList.slice(0, 5) as className}
								<code class="px-1.5 py-0.5 bg-gray-100 text-gray-800 rounded font-mono">
									.{className}
								</code>
							{/each}
							{#if element.classList.length > 5}
								<span class="text-gray-400">+{element.classList.length - 5} more</span>
							{/if}
						</div>
					</div>
				{/if}

				<!-- CSS Selector -->
				<div class="flex items-center gap-2">
					<span class="text-gray-500 w-12">Path:</span>
					<code
						class="px-1.5 py-0.5 bg-gray-100 text-gray-700 rounded font-mono text-[10px] truncate max-w-[200px]"
						title={element.cssSelector}
					>
						{element.cssSelector}
					</code>
				</div>

				<!-- Text Content Preview -->
				{#if element.textContent}
					<div class="flex items-center gap-2">
						<span class="text-gray-500 w-12">Text:</span>
						<span class="text-gray-700 truncate max-w-[200px]" title={element.textContent}>
							"{element.textContent.slice(0, 30)}{element.textContent.length > 30 ? '...' : ''}"
						</span>
					</div>
				{/if}
			</div>
		</div>

		<button
			onclick={onClear}
			class="flex-shrink-0 p-1 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-indigo-100"
			title="Clear selection"
		>
			<svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
			</svg>
		</button>
	</div>
</div>
