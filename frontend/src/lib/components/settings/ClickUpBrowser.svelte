<script lang="ts">
	import {
		getWorkspaces,
		getSpaces,
		getFolders,
		getListsInFolder,
		getFolderlessLists,
		type Team,
		type Space,
		type Folder,
		type List
	} from '$lib/api/clickup';

	interface Props {
		workspaceId: string;
		spaceId: string;
		folderId: string;
		listId: string;
		onchange: (selection: {
			workspaceId: string;
			spaceId: string;
			folderId: string;
			listId: string;
		}) => void;
	}

	let { workspaceId, spaceId, folderId, listId, onchange }: Props = $props();

	let workspaces = $state<Team[]>([]);
	let spaces = $state<Space[]>([]);
	let folders = $state<Folder[]>([]);
	let folderlessLists = $state<List[]>([]);
	let lists = $state<List[]>([]);

	let loading = $state(false);
	let error = $state<string | null>(null);

	// Load workspaces on mount
	$effect(() => {
		loadWorkspaces();
	});

	// Load spaces when workspace changes
	$effect(() => {
		if (workspaceId) {
			loadSpaces(workspaceId);
		} else {
			spaces = [];
			folders = [];
			folderlessLists = [];
			lists = [];
		}
	});

	// Load folders and folderless lists when space changes
	$effect(() => {
		if (spaceId) {
			loadFoldersAndLists(spaceId);
		} else {
			folders = [];
			folderlessLists = [];
			lists = [];
		}
	});

	// Load lists when folder changes
	$effect(() => {
		if (folderId) {
			loadLists(folderId);
		} else {
			lists = [];
		}
	});

	async function loadWorkspaces() {
		loading = true;
		error = null;
		try {
			workspaces = await getWorkspaces();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load workspaces';
		} finally {
			loading = false;
		}
	}

	async function loadSpaces(teamId: string) {
		loading = true;
		error = null;
		try {
			spaces = await getSpaces(teamId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load spaces';
		} finally {
			loading = false;
		}
	}

	async function loadFoldersAndLists(spaceIdValue: string) {
		loading = true;
		error = null;
		try {
			[folders, folderlessLists] = await Promise.all([
				getFolders(spaceIdValue),
				getFolderlessLists(spaceIdValue)
			]);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load folders';
		} finally {
			loading = false;
		}
	}

	async function loadLists(folderIdValue: string) {
		loading = true;
		error = null;
		try {
			lists = await getListsInFolder(folderIdValue);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load lists';
		} finally {
			loading = false;
		}
	}

	function handleWorkspaceChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value;
		onchange({ workspaceId: value, spaceId: '', folderId: '', listId: '' });
	}

	function handleSpaceChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value;
		onchange({ workspaceId, spaceId: value, folderId: '', listId: '' });
	}

	function handleFolderChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value;
		onchange({ workspaceId, spaceId, folderId: value, listId: '' });
	}

	function handleListChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value;
		onchange({ workspaceId, spaceId, folderId, listId: value });
	}

	// Get selected names for breadcrumb
	function getSelectedWorkspace(): string {
		return workspaces.find((w) => w.id === workspaceId)?.name ?? '';
	}

	function getSelectedSpace(): string {
		return spaces.find((s) => s.id === spaceId)?.name ?? '';
	}

	function getSelectedFolder(): string {
		return folders.find((f) => f.id === folderId)?.name ?? '';
	}

	function getSelectedList(): string {
		const allLists = folderId ? lists : folderlessLists;
		return allLists.find((l) => l.id === listId)?.name ?? '';
	}
</script>

<div class="space-y-4">
	<h3 class="text-lg font-medium text-gray-900">ClickUp List Selection</h3>

	{#if error}
		<div class="rounded-md bg-red-50 p-4">
			<p class="text-sm text-red-700">{error}</p>
		</div>
	{/if}

	<!-- Breadcrumb -->
	{#if workspaceId || spaceId || folderId || listId}
		<div class="flex items-center gap-2 text-sm text-gray-500">
			{#if workspaceId}
				<span class="font-medium text-gray-900">{getSelectedWorkspace()}</span>
			{/if}
			{#if spaceId}
				<span>/</span>
				<span class="font-medium text-gray-900">{getSelectedSpace()}</span>
			{/if}
			{#if folderId}
				<span>/</span>
				<span class="font-medium text-gray-900">{getSelectedFolder()}</span>
			{/if}
			{#if listId}
				<span>/</span>
				<span class="font-medium text-indigo-600">{getSelectedList()}</span>
			{/if}
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
		<!-- Workspace Selector -->
		<div>
			<label for="workspace" class="block text-sm font-medium text-gray-700">Workspace</label>
			<select
				id="workspace"
				value={workspaceId}
				onchange={handleWorkspaceChange}
				disabled={loading}
				class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
			>
				<option value="">Select workspace...</option>
				{#each workspaces as workspace}
					<option value={workspace.id}>{workspace.name}</option>
				{/each}
			</select>
		</div>

		<!-- Space Selector -->
		<div>
			<label for="space" class="block text-sm font-medium text-gray-700">Space</label>
			<select
				id="space"
				value={spaceId}
				onchange={handleSpaceChange}
				disabled={!workspaceId || loading}
				class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
			>
				<option value="">Select space...</option>
				{#each spaces as space}
					<option value={space.id}>{space.name}</option>
				{/each}
			</select>
		</div>

		<!-- Folder Selector -->
		<div>
			<label for="folder" class="block text-sm font-medium text-gray-700">Folder (optional)</label>
			<select
				id="folder"
				value={folderId}
				onchange={handleFolderChange}
				disabled={!spaceId || loading}
				class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
			>
				<option value="">No folder (space-level lists)</option>
				{#each folders as folder}
					<option value={folder.id}>{folder.name}</option>
				{/each}
			</select>
		</div>

		<!-- List Selector -->
		<div>
			<label for="list" class="block text-sm font-medium text-gray-700">List</label>
			<select
				id="list"
				value={listId}
				onchange={handleListChange}
				disabled={!spaceId || loading}
				class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
			>
				<option value="">Select list...</option>
				{#if folderId}
					{#each lists as list}
						<option value={list.id}>{list.name}</option>
					{/each}
				{:else}
					{#each folderlessLists as list}
						<option value={list.id}>{list.name}</option>
					{/each}
				{/if}
			</select>
		</div>
	</div>

	{#if loading}
		<div class="flex items-center gap-2 text-sm text-gray-500">
			<svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
			</svg>
			Loading...
		</div>
	{/if}
</div>
