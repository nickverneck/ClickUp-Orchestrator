<script lang="ts">
	import { onMount } from 'svelte';
	import ClickUpBrowser from '$lib/components/settings/ClickUpBrowser.svelte';
	import GitConfig from '$lib/components/settings/GitConfig.svelte';
	import RunnerSettings from '$lib/components/settings/RunnerSettings.svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import { useSettings } from '$lib/stores/settings.svelte';

	const settings = useSettings();
	let sidebarCollapsed = $state(false);

	onMount(() => {
		settings.load();
		// Load sidebar state from localStorage
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}
	});

	// Save sidebar state when it changes
	$effect(() => {
		localStorage.setItem('sidebarCollapsed', String(sidebarCollapsed));
	});

	function handleClickUpChange(selection: {
		workspaceId: string;
		spaceId: string;
		folderId: string;
		listId: string;
	}) {
		settings.set('clickup_workspace_id', selection.workspaceId);
		settings.set('clickup_space_id', selection.spaceId);
		settings.set('clickup_folder_id', selection.folderId);
		settings.set('clickup_list_id', selection.listId);
	}

	function handlePathChange(path: string) {
		settings.set('target_repo_path', path);
	}

	function handleBranchChange(branch: string) {
		settings.set('dev_branch', branch);
	}

	function handleParallelLimitChange(limit: number) {
		settings.set('parallel_limit', limit.toString());
	}

	function handleTriggerStatusChange(status: string) {
		settings.set('trigger_status', status);
	}

	function handleTargetStatusChange(status: string) {
		settings.set('target_status', status);
	}

	function handleAgentPromptChange(prompt: string) {
		settings.set('agent_prompt', prompt);
	}

	async function handleSave() {
		await settings.save();
	}
</script>

<svelte:head>
	<title>Settings - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex h-screen bg-gray-50">
	<Sidebar bind:collapsed={sidebarCollapsed} />

	<main class="flex-1 overflow-auto">
		<div class="mx-auto max-w-4xl px-4 py-8 sm:px-6 lg:px-8">
			<!-- Header -->
			<div class="mb-8">
				<div class="flex items-center justify-between">
					<div>
						<h1 class="text-2xl font-bold text-gray-900">Settings</h1>
						<p class="mt-1 text-sm text-gray-500">
							Configure your ClickUp integration and task runner options
						</p>
					</div>
					<div class="flex items-center gap-4">
						{#if settings.dirty}
							<span class="text-sm text-amber-600">Unsaved changes</span>
						{/if}
					</div>
				</div>
			</div>

		{#if settings.loading}
			<div class="flex items-center justify-center py-12">
				<svg class="h-8 w-8 animate-spin text-indigo-600" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
			</div>
		{:else}
			{#if settings.error}
				<div class="mb-6 rounded-md bg-red-50 p-4">
					<p class="text-sm text-red-700">{settings.error}</p>
				</div>
			{/if}

			<div class="space-y-8">
				<!-- ClickUp Browser Section -->
				<div class="rounded-lg bg-white p-6 shadow">
					<ClickUpBrowser
						workspaceId={settings.get('clickup_workspace_id')}
						spaceId={settings.get('clickup_space_id')}
						folderId={settings.get('clickup_folder_id')}
						listId={settings.get('clickup_list_id')}
						onchange={handleClickUpChange}
					/>
				</div>

				<!-- Git Config Section -->
				<div class="rounded-lg bg-white p-6 shadow">
					<GitConfig
						repoPath={settings.get('target_repo_path')}
						branch={settings.get('dev_branch')}
						onPathChange={handlePathChange}
						onBranchChange={handleBranchChange}
					/>
				</div>

				<!-- Runner Settings Section -->
				<div class="rounded-lg bg-white p-6 shadow">
					<RunnerSettings
						parallelLimit={parseInt(settings.get('parallel_limit') || '1', 10)}
						triggerStatus={settings.get('trigger_status')}
						targetStatus={settings.get('target_status')}
						agentPrompt={settings.get('agent_prompt')}
						listId={settings.get('clickup_list_id') || null}
						onParallelLimitChange={handleParallelLimitChange}
						onTriggerStatusChange={handleTriggerStatusChange}
						onTargetStatusChange={handleTargetStatusChange}
						onAgentPromptChange={handleAgentPromptChange}
					/>
				</div>

				<!-- Save Button -->
				<div class="flex justify-end">
					<button
						type="button"
						onclick={handleSave}
						disabled={!settings.dirty || settings.loading}
						class="inline-flex items-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:cursor-not-allowed disabled:opacity-50"
					>
						{#if settings.loading}
							<svg class="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
						{/if}
						Save Settings
					</button>
				</div>
			</div>
		{/if}
		</div>
	</main>
</div>
