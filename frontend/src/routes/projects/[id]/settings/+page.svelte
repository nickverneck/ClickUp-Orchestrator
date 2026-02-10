<script lang="ts">
	import { page } from '$app/stores';
	import { getProject, updateProject, deleteProject, archiveProject, type Project } from '$lib/api/projects';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	let projectId: number;
	let project = $state<Project | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);
	let success = $state(false);
	let showDeleteConfirm = $state(false);

	// Form fields
	let name = $state('');
	let description = $state('');
	let repoPath = $state('');
	let devBranch = $state('dev');
	let clickupApiKey = $state('');
	let clickupListId = $state('');
	let agentModel = $state('claude');
	let agentPrompt = $state('');
	let parallelLimit = $state(1);

	onMount(async () => {
		projectId = parseInt($page.params.id, 10);
		if (isNaN(projectId)) {
			error = 'Invalid project ID';
			loading = false;
			return;
		}

		try {
			project = await getProject(projectId);
			name = project.name;
			description = project.description || '';
			repoPath = project.repo_path;
			devBranch = project.dev_branch;
			clickupApiKey = project.clickup_api_key || '';
			clickupListId = project.clickup_list_id || '';
			agentModel = project.agent_model;
			agentPrompt = project.agent_prompt || '';
			parallelLimit = project.parallel_limit;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load project';
		} finally {
			loading = false;
		}
	});

	async function handleSave() {
		saving = true;
		error = null;
		success = false;

		try {
			await updateProject(projectId, {
				name,
				description: description || undefined,
				repo_path: repoPath,
				dev_branch: devBranch,
				clickup_api_key: clickupApiKey || undefined,
				clickup_list_id: clickupListId || undefined,
				agent_model: agentModel,
				agent_prompt: agentPrompt || undefined,
				parallel_limit: parallelLimit,
			});
			success = true;
			setTimeout(() => {
				success = false;
			}, 3000);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save settings';
		} finally {
			saving = false;
		}
	}

	async function handleArchive() {
		if (!confirm('Archive this project? You can restore it later.')) return;

		saving = true;
		error = null;

		try {
			await archiveProject(projectId);
			goto('/projects');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to archive project';
			saving = false;
		}
	}

	async function handleDelete() {
		if (!confirm('Delete this project? This action cannot be undone.')) return;

		saving = true;
		error = null;

		try {
			await deleteProject(projectId);
			goto('/projects');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete project';
			saving = false;
		}
	}

	function goBack() {
		goto(`/projects/${projectId}`);
	}
</script>

<svelte:head>
	<title>Project Settings - ClickUp Orchestrator</title>
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<!-- Header -->
	<div class="bg-white shadow">
		<div class="mx-auto max-w-2xl px-4 py-6 sm:px-6 lg:px-8">
			<button onclick={goBack} class="mb-4 inline-flex items-center text-sm text-indigo-600 hover:text-indigo-500">
				<svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
				</svg>
				Back
			</button>
			<h1 class="text-3xl font-bold tracking-tight text-gray-900">Project Settings</h1>
		</div>
	</div>

	<!-- Content -->
	<main class="mx-auto max-w-2xl px-4 py-12 sm:px-6 lg:px-8">
		{#if loading}
			<div class="flex items-center justify-center py-12">
				<svg class="h-12 w-12 animate-spin text-indigo-600" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
			</div>
		{:else if error && !project}
			<div class="rounded-md bg-red-50 p-4">
				<p class="text-sm text-red-700">{error}</p>
			</div>
		{:else if project}
			<div class="space-y-8">
				{#if error}
					<div class="rounded-md bg-red-50 p-4">
						<p class="text-sm text-red-700">{error}</p>
					</div>
				{/if}

				{#if success}
					<div class="rounded-md bg-green-50 p-4">
						<p class="text-sm text-green-700">Settings saved successfully!</p>
					</div>
				{/if}

				<!-- General Settings -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<h2 class="text-lg font-semibold text-gray-900">General</h2>
					<div class="mt-6 space-y-6">
						<div>
							<label class="block text-sm font-semibold text-gray-900">Project Name</label>
							<input
								type="text"
								bind:value={name}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							/>
						</div>

						<div>
							<label class="block text-sm font-semibold text-gray-900">Description</label>
							<textarea
								bind:value={description}
								rows={3}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							></textarea>
						</div>
					</div>
				</div>

				<!-- Git Configuration -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<h2 class="text-lg font-semibold text-gray-900">Git Configuration</h2>
					<div class="mt-6 space-y-6">
						<div>
							<label class="block text-sm font-semibold text-gray-900">Repository Path</label>
							<input
								type="text"
								bind:value={repoPath}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							/>
						</div>

						<div>
							<label class="block text-sm font-semibold text-gray-900">Development Branch</label>
							<input
								type="text"
								bind:value={devBranch}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							/>
						</div>
					</div>
				</div>

				<!-- ClickUp Configuration -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<h2 class="text-lg font-semibold text-gray-900">ClickUp Configuration</h2>
				<div class="mt-6 space-y-6">
					<div>
						<label class="block text-sm font-semibold text-gray-900">API Key</label>
						<input
							type="password"
							bind:value={clickupApiKey}
							placeholder="pk_..."
							class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
						/>
						<p class="mt-1 text-xs text-gray-500">Your ClickUp API key for this project</p>
					</div>
					<div>
						<label class="block text-sm font-semibold text-gray-900">List ID</label>
						<input
							type="text"
							bind:value={clickupListId}
							placeholder="Optional"
							class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
						/>
					</div>
				</div>
				</div>

				<!-- Agent Configuration -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<h2 class="text-lg font-semibold text-gray-900">Agent Configuration</h2>
					<div class="mt-6 space-y-6">
						<div>
							<label class="block text-sm font-semibold text-gray-900">Model</label>
							<select
								bind:value={agentModel}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							>
								<option value="claude">Claude</option>
								<option value="gpt4">GPT-4</option>
							</select>
						</div>

						<div>
							<label class="block text-sm font-semibold text-gray-900">Custom Prompt</label>
							<textarea
								bind:value={agentPrompt}
								placeholder="Optional instructions for the agent"
								rows={4}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							></textarea>
						</div>

						<div>
							<label class="block text-sm font-semibold text-gray-900">Parallel Task Limit</label>
							<input
								type="number"
								bind:value={parallelLimit}
								min="1"
								max="10"
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							/>
						</div>
					</div>
				</div>

				<!-- Danger Zone -->
				<div class="rounded-lg border border-red-200 bg-red-50 p-6">
					<h2 class="text-lg font-semibold text-red-900">Danger Zone</h2>
					<div class="mt-6 space-y-3">
						<button
							onclick={handleArchive}
							disabled={saving}
							class="block w-full rounded-md border border-red-300 bg-white px-4 py-2 text-sm font-semibold text-red-700 hover:bg-red-50 disabled:opacity-50"
						>
							Archive Project
						</button>
						<button
							onclick={handleDelete}
							disabled={saving}
							class="block w-full rounded-md bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-500 disabled:opacity-50"
						>
							Delete Project
						</button>
					</div>
				</div>

				<!-- Save Button -->
				<div class="flex gap-3">
					<button
						onclick={goBack}
						class="flex-1 rounded-md border border-gray-300 px-4 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50"
					>
						Cancel
					</button>
					<button
						onclick={handleSave}
						disabled={saving}
						class="flex-1 rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:opacity-50"
					>
						{#if saving}
							Saving...
						{:else}
							Save Changes
						{/if}
					</button>
				</div>
			</div>
		{/if}
	</main>
</div>
