<script lang="ts">
	import type { EditorTab } from '$lib/types/editor';
	import EditorTabComponent from './EditorTab.svelte';
	import CodeEditor from './CodeEditor.svelte';

	interface Props {
		tabs: EditorTab[];
		activeTabId: string | null;
		onTabSelect: (id: string) => void;
		onTabClose: (id: string) => void;
		onContentChange: (id: string, content: string) => void;
		onSave: (id: string) => void;
	}

	let { tabs, activeTabId, onTabSelect, onTabClose, onContentChange, onSave }: Props = $props();

	let activeTab = $derived(tabs.find((t) => t.id === activeTabId));

	function handleKeydown(event: KeyboardEvent) {
		// Ctrl/Cmd + S to save
		if ((event.ctrlKey || event.metaKey) && event.key === 's') {
			event.preventDefault();
			if (activeTabId) {
				onSave(activeTabId);
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full">
	<!-- Tab Bar -->
	{#if tabs.length > 0}
		<div class="flex items-center bg-gray-900 border-b border-gray-700 overflow-x-auto">
			{#each tabs as tab (tab.id)}
				<EditorTabComponent
					{tab}
					isActive={tab.id === activeTabId}
					onSelect={() => onTabSelect(tab.id)}
					onClose={() => onTabClose(tab.id)}
				/>
			{/each}
		</div>
	{/if}

	<!-- Editor Content -->
	<div class="flex-1 overflow-hidden">
		{#if activeTab}
			<CodeEditor
				content={activeTab.content}
				language={activeTab.language}
				onContentChange={(content) => onContentChange(activeTab.id, content)}
			/>
		{:else}
			<div class="h-full flex items-center justify-center bg-gray-900">
				<div class="text-center text-gray-500">
					<svg class="w-16 h-16 mx-auto mb-4 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
						/>
					</svg>
					<p class="text-lg font-medium">No file open</p>
					<p class="text-sm mt-1">Select a file from the explorer to start editing</p>
				</div>
			</div>
		{/if}
	</div>
</div>
