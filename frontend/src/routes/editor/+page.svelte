<script lang="ts">
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import FileExplorer from '$lib/components/editor/FileExplorer.svelte';
	import EditorPanel from '$lib/components/editor/EditorPanel.svelte';
	import type { FileNode, EditorTab } from '$lib/types/editor';
	import { getSettings } from '$lib/api/settings';

	let sidebarCollapsed = $state(false);

	// File explorer state
	let fileTree = $state<FileNode[]>([]);
	let expandedFolders = $state<Set<string>>(new Set());
	let rootPath = $state('');
	let loading = $state(true);

	// Editor state
	let openTabs = $state<EditorTab[]>([]);
	let activeTabId = $state<string | null>(null);
	let unsavedChanges = $state<Set<string>>(new Set());

	onMount(async () => {
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}

		// Load root path from settings
		try {
			const settings = await getSettings();
			if (settings.target_repo_path) {
				rootPath = settings.target_repo_path;
			}
		} catch (e) {
			console.error('Failed to load target_repo_path:', e);
		}
		loading = false;
	});

	$effect(() => {
		localStorage.setItem('sidebarCollapsed', String(sidebarCollapsed));
	});

	function handleFileSelect(path: string, name: string) {
		// Check if already open
		const existingTab = openTabs.find((t) => t.path === path);
		if (existingTab) {
			activeTabId = existingTab.id;
			return;
		}

		// Open new tab
		const newTab: EditorTab = {
			id: crypto.randomUUID(),
			path,
			name,
			content: '',
			language: getLanguageFromPath(path),
			isDirty: false
		};
		openTabs = [...openTabs, newTab];
		activeTabId = newTab.id;
	}

	function handleTabClose(tabId: string) {
		const tabIndex = openTabs.findIndex((t) => t.id === tabId);
		openTabs = openTabs.filter((t) => t.id !== tabId);
		unsavedChanges.delete(tabId);
		unsavedChanges = new Set(unsavedChanges);

		// Update active tab
		if (activeTabId === tabId) {
			if (openTabs.length > 0) {
				const newIndex = Math.min(tabIndex, openTabs.length - 1);
				activeTabId = openTabs[newIndex]?.id ?? null;
			} else {
				activeTabId = null;
			}
		}
	}

	function handleContentChange(tabId: string, content: string) {
		openTabs = openTabs.map((t) => (t.id === tabId ? { ...t, content, isDirty: true } : t));
		unsavedChanges.add(tabId);
		unsavedChanges = new Set(unsavedChanges);
	}

	function handleSave(tabId: string) {
		const tab = openTabs.find((t) => t.id === tabId);
		if (!tab) return;

		// TODO: Implement save via API
		console.log('Saving:', tab.path, tab.content);
		openTabs = openTabs.map((t) => (t.id === tabId ? { ...t, isDirty: false } : t));
		unsavedChanges.delete(tabId);
		unsavedChanges = new Set(unsavedChanges);
	}

	function getLanguageFromPath(path: string): string {
		const ext = path.split('.').pop()?.toLowerCase();
		const langMap: Record<string, string> = {
			ts: 'typescript',
			tsx: 'typescript',
			js: 'javascript',
			jsx: 'javascript',
			svelte: 'html',
			html: 'html',
			css: 'css',
			scss: 'scss',
			json: 'json',
			md: 'markdown',
			rs: 'rust',
			py: 'python',
			go: 'go',
			yaml: 'yaml',
			yml: 'yaml',
			toml: 'toml',
			sql: 'sql',
			sh: 'shell',
			bash: 'shell'
		};
		return langMap[ext ?? ''] || 'plaintext';
	}

	function handleFolderToggle(path: string) {
		if (expandedFolders.has(path)) {
			expandedFolders.delete(path);
		} else {
			expandedFolders.add(path);
		}
		expandedFolders = new Set(expandedFolders);
	}
</script>

<svelte:head>
	<title>File Editor - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex h-screen bg-gray-50">
	<Sidebar bind:collapsed={sidebarCollapsed} />

	<main class="flex-1 flex flex-col overflow-hidden">
		<!-- Header -->
		<div class="border-b border-gray-200 bg-white px-6 py-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<h1 class="text-xl font-bold text-gray-900">File Editor</h1>
					<span
						class="text-xs font-medium bg-amber-100 text-amber-700 px-2 py-1 rounded-full"
					>
						EXPERIMENTAL
					</span>
				</div>
				{#if rootPath}
					<div class="flex items-center gap-2 text-sm">
						<svg class="w-4 h-4 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
						</svg>
						<span class="font-mono text-gray-600 truncate max-w-md">{rootPath}</span>
					</div>
				{/if}
			</div>
		</div>

		<!-- Main Content -->
		<div class="flex-1 flex overflow-hidden">
			{#if loading}
				<div class="flex-1 flex items-center justify-center">
					<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
				</div>
			{:else if !rootPath}
				<div class="flex-1 flex items-center justify-center">
					<div class="text-center">
						<svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
						</svg>
						<h3 class="mt-2 text-sm font-medium text-gray-900">No project configured</h3>
						<p class="mt-1 text-sm text-gray-500">
							Configure a target repository in Settings to use the file editor.
						</p>
						<div class="mt-6">
							<a
								href="/settings"
								class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
							>
								Go to Settings
							</a>
						</div>
					</div>
				</div>
			{:else}
				<!-- File Explorer - Left Side -->
				<div class="w-64 border-r border-gray-200 bg-gray-900 flex flex-col">
					<FileExplorer
						{rootPath}
						{expandedFolders}
						onFileSelect={handleFileSelect}
						onFolderToggle={handleFolderToggle}
					/>
				</div>

				<!-- Editor Panel - Right Side -->
				<div class="flex-1 flex flex-col bg-gray-800">
					<EditorPanel
						tabs={openTabs}
						{activeTabId}
						onTabSelect={(id) => (activeTabId = id)}
						onTabClose={handleTabClose}
						onContentChange={handleContentChange}
						onSave={handleSave}
					/>
				</div>
			{/if}
		</div>
	</main>
</div>
