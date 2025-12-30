<script lang="ts">
	import { onMount } from 'svelte';
	import type { FileNode } from '$lib/types/editor';
	import { getFileTree } from '$lib/api/files';
	import FileTreeNode from './FileTreeNode.svelte';

	interface Props {
		rootPath: string;
		expandedFolders: Set<string>;
		onFileSelect: (path: string, name: string) => void;
		onFolderToggle: (path: string) => void;
	}

	let { rootPath, expandedFolders, onFileSelect, onFolderToggle }: Props = $props();

	let fileTree = $state<FileNode[]>([]);
	let loading = $state(true);
	let error = $state('');

	async function loadTree() {
		if (!rootPath) return;

		loading = true;
		error = '';

		try {
			fileTree = await getFileTree(rootPath);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load file tree';
			fileTree = [];
		} finally {
			loading = false;
		}
	}

	onMount(loadTree);

	$effect(() => {
		if (rootPath) {
			loadTree();
		}
	});
</script>

<div class="h-full flex flex-col text-gray-300">
	<!-- Header -->
	<div class="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider flex items-center justify-between border-b border-gray-700">
		<span>Explorer</span>
		<button
			onclick={loadTree}
			class="p-1 hover:bg-gray-700 rounded transition-colors"
			title="Refresh file tree"
		>
			<svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
				/>
			</svg>
		</button>
	</div>

	<!-- Tree Content -->
	<div class="flex-1 overflow-auto py-2">
		{#if loading}
			<div class="flex items-center justify-center py-8">
				<svg class="animate-spin h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24">
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
			</div>
		{:else if error}
			<div class="px-3 py-4 text-center">
				<svg class="w-8 h-8 mx-auto text-red-400 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
					/>
				</svg>
				<p class="text-sm text-red-400">{error}</p>
				<button
					onclick={loadTree}
					class="mt-2 text-xs text-indigo-400 hover:text-indigo-300"
				>
					Try again
				</button>
			</div>
		{:else if fileTree.length === 0}
			<div class="px-3 py-4 text-center text-gray-500 text-sm">
				No files found
			</div>
		{:else}
			{#each fileTree as node}
				<FileTreeNode
					{node}
					depth={0}
					{expandedFolders}
					{onFileSelect}
					{onFolderToggle}
				/>
			{/each}
		{/if}
	</div>
</div>
