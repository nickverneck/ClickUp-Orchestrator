<script lang="ts">
	import type { FileNode } from '$lib/types/editor';
	import Self from './FileTreeNode.svelte';

	interface Props {
		node: FileNode;
		depth: number;
		expandedFolders: Set<string>;
		onFileSelect: (path: string, name: string) => void;
		onFolderToggle: (path: string) => void;
	}

	let { node, depth, expandedFolders, onFileSelect, onFolderToggle }: Props = $props();

	let isExpanded = $derived(expandedFolders.has(node.path));

	function handleClick() {
		if (node.isDirectory) {
			onFolderToggle(node.path);
		} else {
			onFileSelect(node.path, node.name);
		}
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
			go: 'text-cyan-400',
			toml: 'text-gray-400',
			yaml: 'text-red-400',
			yml: 'text-red-400',
			sql: 'text-blue-300',
			sh: 'text-green-500',
			bash: 'text-green-500'
		};
		return iconMap[ext ?? ''] || 'text-gray-400';
	}
</script>

<div>
	<button
		onclick={handleClick}
		class="w-full flex items-center gap-1.5 px-2 py-1 hover:bg-gray-700/50 text-left text-sm transition-colors"
		style="padding-left: {depth * 12 + 8}px"
	>
		<!-- Expand/Collapse Arrow for Directories -->
		{#if node.isDirectory}
			<svg
				class="w-3 h-3 text-gray-500 flex-shrink-0 transition-transform {isExpanded ? 'rotate-90' : ''}"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
			</svg>
		{:else}
			<div class="w-3"></div>
		{/if}

		<!-- Icon -->
		{#if node.isDirectory}
			{#if isExpanded}
				<svg class="w-4 h-4 text-amber-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"
					/>
				</svg>
			{:else}
				<svg class="w-4 h-4 text-amber-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
					/>
				</svg>
			{/if}
		{:else}
			<svg class="w-4 h-4 {getFileIcon(node.name)} flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
				/>
			</svg>
		{/if}

		<!-- Name -->
		<span class="truncate {node.isDirectory ? 'text-gray-200' : 'text-gray-300'}">
			{node.name}
		</span>
	</button>

	<!-- Children (if directory is expanded) -->
	{#if node.isDirectory && isExpanded && node.children}
		{#each node.children as child}
			<Self
				node={child}
				depth={depth + 1}
				{expandedFolders}
				{onFileSelect}
				{onFolderToggle}
			/>
		{/each}
	{/if}
</div>
