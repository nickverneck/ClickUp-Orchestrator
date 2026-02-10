<script lang="ts">
	import { listFolders, type FolderListResponse } from '$lib/api/projects';

	interface Props {
		onSelect: (path: string) => void;
		selectedPath?: string;
	}

	let { onSelect, selectedPath = '' }: Props = $props();

	let loading = $state(false);
	let error = $state<string | null>(null);
	let folderData = $state<FolderListResponse | null>(null);
	let currentPath = $state<string | null>(null);

	async function loadFolders(path?: string) {
		loading = true;
		error = null;
		try {
			const data = await listFolders(path);
			folderData = data;
			currentPath = data.current_path;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load folders';
		} finally {
			loading = false;
		}
	}

	function navigateTo(folderPath: string) {
		loadFolders(folderPath);
	}

	function goUp() {
		if (folderData?.can_go_up && currentPath) {
			const parent = currentPath.substring(0, currentPath.lastIndexOf('/'));
			navigateTo(parent || folderData.base_path);
		}
	}

	function selectFolder() {
		if (currentPath) {
			onSelect(currentPath);
		}
	}

	// Load initial folders on mount
	loadFolders();
</script>

<div class="w-full space-y-4">
	{#if error}
		<div class="rounded-md bg-red-50 p-3">
			<p class="text-sm text-red-700">{error}</p>
		</div>
	{/if}

	<!-- Current Path Display -->
	<div class="rounded-md border border-gray-200 bg-gray-50 px-3 py-2">
		<p class="text-xs font-medium text-gray-600">Current Path:</p>
		<p class="mt-1 break-all text-sm font-mono text-gray-900">{currentPath || 'Loading...'}</p>
	</div>

	<!-- Navigation Breadcrumbs -->
	{#if folderData}
		<div class="flex items-center gap-2">
			{#if folderData.can_go_up}
				<button
					onclick={goUp}
					class="inline-flex items-center rounded-md border border-gray-300 bg-white px-3 py-1 text-sm font-medium text-gray-700 hover:bg-gray-50"
				>
					<svg class="mr-1 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16l-4-4m0 0l4-4m-4 4h18" />
					</svg>
					Up
				</button>
			{/if}
			<span class="text-xs text-gray-500">{folderData.folders.length} folder{folderData.folders.length !== 1 ? 's' : ''}</span>
		</div>
	{/if}

	<!-- Folder List -->
	<div class="max-h-96 overflow-y-auto rounded-md border border-gray-200 bg-white">
		{#if loading}
			<div class="flex items-center justify-center py-8">
				<div class="animate-spin text-gray-400">
					<svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
					</svg>
				</div>
			</div>
		{:else if folderData && folderData.folders.length > 0}
			<div class="divide-y divide-gray-200 border border-gray-200 rounded-md overflow-hidden">
				{#each folderData.folders as folder (folder.path)}
					<button
						onclick={() => navigateTo(folder.path)}
						class="flex w-full cursor-pointer items-center gap-3 px-4 py-3 text-left hover:bg-indigo-50 transition-colors"
					>
						<svg class="h-5 w-5 flex-shrink-0 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
							<path d="M4 4a2 2 0 00-2 2v4a2 2 0 002 2V6h10a2 2 0 00-2-2H4zm2 6a2 2 0 012-2h8a2 2 0 012 2v4a2 2 0 01-2 2H8a2 2 0 01-2-2v-4zm6 4a2 2 0 100-4 2 2 0 000 4z" />
						</svg>
						<span class="flex-1 text-sm font-medium text-gray-900">
							{folder.name}
						</span>
						{#if folder.is_git_repo}
							<span class="inline-flex items-center rounded-full bg-green-50 px-2.5 py-0.5 text-xs font-medium text-green-700">
								Git
							</span>
						{/if}
					</button>
				{/each}
			</div>
		{:else if folderData}
			<div class="flex items-center justify-center py-8">
				<p class="text-sm text-gray-500">No folders found</p>
			</div>
		{/if}
	</div>

	<!-- Selected Path Display -->
	{#if selectedPath && selectedPath !== currentPath}
		<div class="rounded-md border border-amber-200 bg-amber-50 p-3">
			<p class="text-xs font-medium text-amber-800">Previously Selected:</p>
			<p class="mt-1 break-all text-sm font-mono text-amber-900">{selectedPath}</p>
		</div>
	{/if}

	<!-- Select Button -->
	<div class="flex gap-2">
		<button
			onclick={selectFolder}
			disabled={loading || !currentPath}
			class="flex-1 rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:opacity-50"
		>
			Select This Folder
		</button>
	</div>
</div>
