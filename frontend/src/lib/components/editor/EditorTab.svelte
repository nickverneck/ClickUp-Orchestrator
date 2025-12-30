<script lang="ts">
	import type { EditorTab } from '$lib/types/editor';

	interface Props {
		tab: EditorTab;
		isActive: boolean;
		onSelect: () => void;
		onClose: () => void;
	}

	let { tab, isActive, onSelect, onClose }: Props = $props();

	function handleClose(event: MouseEvent) {
		event.stopPropagation();
		onClose();
	}

	function getFileIcon(name: string): string {
		const ext = name.split('.').pop()?.toLowerCase();
		const iconMap: Record<string, string> = {
			ts: 'text-blue-400',
			tsx: 'text-blue-400',
			js: 'text-yellow-400',
			jsx: 'text-yellow-400',
			svelte: 'text-orange-400',
			html: 'text-orange-500',
			css: 'text-blue-500',
			scss: 'text-pink-400',
			json: 'text-yellow-500',
			md: 'text-gray-400',
			rs: 'text-orange-600',
			py: 'text-green-400',
			go: 'text-cyan-400'
		};
		return iconMap[ext ?? ''] || 'text-gray-400';
	}
</script>

<div
	class="group flex items-center gap-2 px-3 py-2 border-r border-gray-700 text-sm {isActive
		? 'bg-gray-800 text-gray-100'
		: 'bg-gray-900 text-gray-400 hover:bg-gray-800 hover:text-gray-200'} transition-colors min-w-0"
>
	<button onclick={onSelect} class="flex items-center gap-2 flex-1 min-w-0">
		<!-- File Icon -->
		<svg class="w-4 h-4 flex-shrink-0 {getFileIcon(tab.name)}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
			/>
		</svg>

		<!-- File Name -->
		<span class="truncate max-w-[150px]">
			{tab.name}
		</span>
	</button>

	<!-- Dirty Indicator -->
	{#if tab.isDirty}
		<span class="w-2 h-2 rounded-full bg-amber-400 flex-shrink-0"></span>
	{/if}

	<!-- Close Button -->
	<button
		onclick={handleClose}
		class="p-0.5 rounded hover:bg-gray-600 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
		title="Close"
	>
		<svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
		</svg>
	</button>
</div>
