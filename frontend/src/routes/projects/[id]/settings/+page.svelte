<script lang="ts">
	import { page } from '$app/stores';
	import { getProject, updateProject, deleteProject, archiveProject, type Project } from '$lib/api/projects';
	import { getWorkspaces } from '$lib/api/clickup';
	import { getOpenCodeModels, type AgentModel } from '$lib/api/agents';
	import ClickUpBrowser from '$lib/components/settings/ClickUpBrowser.svelte';
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
	let clickupWorkspaceId = $state('');
	let clickupSpaceId = $state('');
	let clickupFolderId = $state('');
	let clickupListId = $state('');
	let agentModel = $state('claude');
	let opencodeModel = $state('');
	let opencodeModels = $state<AgentModel[]>([]);
	let opencodeModelsLoading = $state(false);
	let agentPrompt = $state('');
	let parallelLimit = $state(1);

	// API key validation
	let apiKeyValid = $state(false);
	let apiKeyError = $state<string | null>(null);
	let apiKeyChecked = $state(false);

	onMount(async () => {
		if (!$page.params.id) {
			error = 'Invalid project ID';
			loading = false;
			return;
		}
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
			clickupWorkspaceId = project.clickup_workspace_id || '';
			clickupSpaceId = project.clickup_space_id || '';
			clickupFolderId = project.clickup_folder_id || '';
			clickupListId = project.clickup_list_id || '';
			agentModel = project.agent_model;
			opencodeModel = project.opencode_model || 'opencode/kimi-k2.5-free';
			agentPrompt = project.agent_prompt || '';
			parallelLimit = project.parallel_limit;

			// If project already has an API key, validate it
			if (clickupApiKey) {
				await validateApiKey();
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load project';
		} finally {
			loading = false;
		}
	});

	async function validateApiKey() {
		if (!clickupApiKey.trim()) {
			apiKeyValid = false;
			apiKeyError = null;
			apiKeyChecked = false;
			return;
		}

		apiKeyChecked = false;
		try {
			await getWorkspaces(clickupApiKey);
			apiKeyValid = true;
			apiKeyError = null;
		} catch (e) {
			apiKeyValid = false;
			apiKeyError = 'Invalid API key';
		} finally {
			apiKeyChecked = true;
		}
	}

	async function loadOpenCodeModels() {
		opencodeModelsLoading = true;
		try {
			opencodeModels = await getOpenCodeModels();
		} catch (e) {
			opencodeModels = [];
		} finally {
			opencodeModelsLoading = false;
		}
	}

	$effect(() => {
		if (agentModel === 'opencode') {
			loadOpenCodeModels();
		}
	});

	function handleClickUpChange(selection: {
		workspaceId: string;
		spaceId: string;
		folderId: string;
		listId: string;
	}) {
		clickupWorkspaceId = selection.workspaceId;
		clickupSpaceId = selection.spaceId;
		clickupFolderId = selection.folderId;
		clickupListId = selection.listId;
	}

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
				clickup_workspace_id: clickupWorkspaceId || undefined,
				clickup_space_id: clickupSpaceId || undefined,
				clickup_folder_id: clickupFolderId || undefined,
				clickup_list_id: clickupListId || undefined,
				agent_model: agentModel,
				opencode_model: agentModel === 'opencode' ? opencodeModel || undefined : undefined,
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

<div class="flex-1 overflow-auto">
	<!-- Header -->
	<div class="bg-white shadow">
		<div class="mx-auto max-w-2xl px-4 py-6 sm:px-6 lg:px-8">
			<h1 class="text-2xl font-semibold text-gray-900">Project Settings</h1>
		</div>
	</div>

	<!-- Content -->
	<main class="mx-auto max-w-2xl px-4 py-8 sm:px-6 lg:px-8">
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
							<label for="project-name" class="block text-sm font-semibold text-gray-900">Project Name</label>
							<input
								id="project-name"
								type="text"
								bind:value={name}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							/>
						</div>

						<div>
							<label for="description" class="block text-sm font-semibold text-gray-900">Description</label>
							<textarea
								id="description"
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
							<label for="repo-path" class="block text-sm font-semibold text-gray-900">Repository Path</label>
							<input
								id="repo-path"
								type="text"
								bind:value={repoPath}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							/>
						</div>

						<div>
							<label for="dev-branch" class="block text-sm font-semibold text-gray-900">Development Branch</label>
							<input
								id="dev-branch"
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
							<label for="api-key" class="block text-sm font-semibold text-gray-900">API Key</label>
							<input
								id="api-key"
								type="password"
								bind:value={clickupApiKey}
								onchange={validateApiKey}
								placeholder="pk_..."
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							/>
							{#if apiKeyError}
								<p class="mt-1 text-sm text-red-600">{apiKeyError}</p>
							{:else if clickupApiKey && apiKeyValid}
								<p class="mt-1 text-sm text-green-600">✓ API key is valid</p>
							{:else}
								<p class="mt-1 text-xs text-gray-500">Your ClickUp API key for this project</p>
							{/if}
						</div>

						{#if clickupApiKey && apiKeyValid}
							<div class="rounded-lg border border-gray-200 bg-gray-50 p-4">
								<ClickUpBrowser
									apiKey={clickupApiKey}
									workspaceId={clickupWorkspaceId}
									spaceId={clickupSpaceId}
									folderId={clickupFolderId}
									listId={clickupListId}
									compact={true}
									onchange={handleClickUpChange}
								/>
							</div>
						{:else if clickupApiKey && !apiKeyChecked}
							<div class="rounded-md bg-blue-50 p-4">
								<p class="text-sm text-blue-700">Validating API key...</p>
							</div>
						{:else if !clickupApiKey}
							<div class="rounded-md bg-blue-50 p-4">
								<p class="text-sm text-blue-700">Add a ClickUp API key above to browse your ClickUp workspace</p>
							</div>
						{/if}
					</div>
				</div>

				<!-- Agent Configuration -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<h2 class="text-lg font-semibold text-gray-900">Agent Configuration</h2>
					<div class="mt-6 space-y-6">
						<div>
							<label for="agent-model" class="block text-sm font-semibold text-gray-900">Model</label>
							<select
								id="agent-model"
								bind:value={agentModel}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							>
								<option value="claude">Claude</option>
								<option value="codex">Codex</option>
								<option value="gemini">Gemini</option>
								<option value="opencode">OpenCode</option>
							</select>
						</div>

						{#if agentModel === 'opencode'}
							<div>
								<label for="opencode-model" class="block text-sm font-semibold text-gray-900">OpenCode Model</label>
								{#if opencodeModelsLoading}
									<p class="mt-2 text-sm text-gray-500">Loading models...</p>
								{:else if opencodeModels.length > 0}
									<select
										id="opencode-model"
										bind:value={opencodeModel}
										class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
									>
										{#each opencodeModels as model}
											<option value={model.id}>{model.name} ({model.provider})</option>
										{/each}
									</select>
								{:else}
									<input
										id="opencode-model"
										type="text"
										bind:value={opencodeModel}
										placeholder="opencode/kimi-k2.5-free"
										class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
									/>
								{/if}
								<p class="mt-1 text-xs text-gray-500">Select the model for OpenCode to use</p>
							</div>
						{/if}

						<div>
							<label for="agent-prompt" class="block text-sm font-semibold text-gray-900">Custom Prompt</label>
							<textarea
								id="agent-prompt"
								bind:value={agentPrompt}
								placeholder="Optional instructions for the agent"
								rows={4}
								class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
							></textarea>
						</div>

						<div>
							<label for="parallel-limit" class="block text-sm font-semibold text-gray-900">Parallel Task Limit</label>
							<input
								id="parallel-limit"
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
