<script lang="ts">
	import { createProject, cloneProjectFromGithub, type CreateProjectRequest } from '$lib/api/projects';
	import { goto } from '$app/navigation';

	type CreationType = 'new' | 'clone' | null;

	let step = $state<1 | 2 | 3>(1);
	let creationType = $state<CreationType>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	// Form data
	let name = $state('');
	let description = $state('');
	let repoPath = $state('');
	let githubUrl = $state('');
	let targetPath = $state('');
	let devBranch = $state('dev');
	let clickupListId = $state('');
	let agentModel = $state('claude');
	let agentPrompt = $state('');
	let parallelLimit = $state(1);
	let clickupApiKey = $state('');
	let apiKeyError = $state<string | null>(null);
	let apiKeyValid = $state(false);

	function selectType(type: CreationType) {
		creationType = type;
		step = 2;
	}

	function goBack() {
		if (step === 1) {
			goto('/projects');
		} else {
			step = (step - 1) as 1 | 2 | 3;
		}
	}

	async function handleCreateProject() {
		if (!name.trim()) {
			error = 'Project name is required';
			return;
		}

		loading = true;
		error = null;

		try {
			const request: CreateProjectRequest = {
				name: name.trim(),
				description: description.trim() || undefined,
				repo_path: repoPath.trim(),
				dev_branch: devBranch.trim() || 'dev',
				clickup_api_key: clickupApiKey || undefined,
				agent_model: agentModel,
				agent_prompt: agentPrompt || undefined,
				clickup_list_id: clickupListId || undefined,
				parallel_limit: parallelLimit,
			};

			const result = await createProject(request);
			goto(`/projects/${result.id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create project';
		} finally {
			loading = false;
		}
	}

	async function handleCloneProject() {
		if (!name.trim()) {
			error = 'Project name is required';
			return;
		}
		if (!githubUrl.trim()) {
			error = 'GitHub URL is required';
			return;
		}
		if (!targetPath.trim()) {
			error = 'Target path is required';
			return;
		}

		loading = true;
		error = null;

		try {
			const request = {
				name: name.trim(),
				description: description.trim() || undefined,
				github_url: githubUrl.trim(),
				target_path: targetPath.trim(),
				dev_branch: devBranch.trim() || 'dev',
				clickup_api_key: clickupApiKey || undefined,
				agent_model: agentModel,
				agent_prompt: agentPrompt || undefined,
				clickup_list_id: clickupListId || undefined,
				parallel_limit: parallelLimit,
			};

			const result = await cloneProjectFromGithub(request);
			goto(`/projects/${result.id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to clone project';
		} finally {
			loading = false;
		}
	}

	async function validateApiKey() {
		if (!clickupApiKey.trim()) {
			apiKeyValid = false;
			apiKeyError = null;
			return;
		}

		try {
			const response = await fetch(`/api/clickup/workspaces?api_key=${encodeURIComponent(clickupApiKey)}`);
			if (response.ok) {
				apiKeyValid = true;
				apiKeyError = null;
			} else {
				apiKeyValid = false;
				apiKeyError = 'Invalid API key';
			}
		} catch (e) {
			apiKeyValid = false;
			apiKeyError = 'Failed to validate API key';
		}
	}
</script>

<svelte:head>
	<title>Create Project - ClickUp Orchestrator</title>
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
			<h1 class="text-3xl font-bold tracking-tight text-gray-900">Create New Project</h1>
			<div class="mt-4 flex gap-2">
				{#each [1, 2, 3] as s (s)}
					<div class="flex items-center">
						<div
							class="flex h-8 w-8 items-center justify-center rounded-full {step >= s
								? 'bg-indigo-600 text-white'
								: 'bg-gray-300 text-gray-600'}"
						>
							{s}
						</div>
						{#if s < 3}
							<div class="mx-2 h-1 w-8 {step > s ? 'bg-indigo-600' : 'bg-gray-300'}"></div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	</div>

	<!-- Content -->
	<main class="mx-auto max-w-2xl px-4 py-12 sm:px-6 lg:px-8">
		{#if error}
			<div class="mb-6 rounded-md bg-red-50 p-4">
				<p class="text-sm text-red-700">{error}</p>
			</div>
		{/if}

		{#if step === 1}
			<!-- Step 1: Choose type -->
			<div class="space-y-4">
				<h2 class="text-lg font-semibold text-gray-900">How would you like to create your project?</h2>

				<button
					onclick={() => selectType('new')}
					class="w-full rounded-lg border-2 border-gray-200 p-6 text-left transition-all hover:border-indigo-500 hover:bg-indigo-50"
				>
					<h3 class="font-semibold text-gray-900">Create from Existing Folder</h3>
					<p class="mt-2 text-sm text-gray-600">Use an existing git repository on your machine</p>
				</button>

				<button
					onclick={() => selectType('clone')}
					class="w-full rounded-lg border-2 border-gray-200 p-6 text-left transition-all hover:border-indigo-500 hover:bg-indigo-50"
				>
					<h3 class="font-semibold text-gray-900">Clone from GitHub</h3>
					<p class="mt-2 text-sm text-gray-600">Clone a public GitHub repository to your machine</p>
				</button>
			</div>
		{:else if step === 2}
			<!-- Step 2: Repository setup -->
			<div class="space-y-6">
				<div>
					<label class="block text-sm font-semibold text-gray-900">Project Name *</label>
					<input
						type="text"
						bind:value={name}
						placeholder="My Project"
						class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
					/>
				</div>

				<div>
					<label class="block text-sm font-semibold text-gray-900">Description</label>
					<textarea
						bind:value={description}
						placeholder="Optional project description"
						rows={3}
						class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
					></textarea>
				</div>

				{#if creationType === 'new'}
					<div>
						<label class="block text-sm font-semibold text-gray-900">Repository Path *</label>
						<input
							type="text"
							bind:value={repoPath}
							placeholder="/path/to/repo"
							class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
						/>
						<p class="mt-1 text-xs text-gray-500">Full path to existing git repository</p>
					</div>
				{:else if creationType === 'clone'}
					<div>
						<label class="block text-sm font-semibold text-gray-900">GitHub Repository URL *</label>
						<input
							type="text"
							bind:value={githubUrl}
							placeholder="https://github.com/user/repo"
							class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
						/>
					</div>

					<div>
						<label class="block text-sm font-semibold text-gray-900">Clone Target Path *</label>
						<input
							type="text"
							bind:value={targetPath}
							placeholder="/path/to/clone"
							class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
						/>
						<p class="mt-1 text-xs text-gray-500">Where to clone the repository</p>
					</div>
				{/if}

				<div>
					<label class="block text-sm font-semibold text-gray-900">Development Branch</label>
					<input
						type="text"
						bind:value={devBranch}
						placeholder="dev"
						class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
					/>
				</div>

				<div class="flex gap-3">
					<button
						onclick={goBack}
						class="flex-1 rounded-md border border-gray-300 px-4 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50"
					>
						Back
					</button>
					<button
						onclick={() => {
							step = 3;
						}}
						class="flex-1 rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500"
					>
						Next
					</button>
				</div>
			</div>
		{:else if step === 3}
			<!-- Step 3: Configuration -->
			<div class="space-y-6">
				<div>
					<label class="block text-sm font-semibold text-gray-900">
						ClickUp API Key
						<span class="font-normal text-gray-500">(optional)</span>
					</label>
					<input
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
					{/if}
				</div>

				{#if clickupApiKey && apiKeyValid}
					<div>
						<label class="block text-sm font-semibold text-gray-900">ClickUp List ID</label>
						<input
							type="text"
							bind:value={clickupListId}
							placeholder="Optional"
							class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
						/>
					</div>
				{:else}
					<div class="rounded-md bg-blue-50 p-4">
						<p class="text-sm text-blue-700">
							{#if !clickupApiKey}
								Add a ClickUp API key above to configure your ClickUp workspace
							{:else}
								Validating API key...
							{/if}
						</p>
					</div>
				{/if}

				<div>
					<label class="block text-sm font-semibold text-gray-900">Agent Model</label>
					<select
						bind:value={agentModel}
						class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
					>
						<option value="claude">Claude</option>
						<option value="gpt4">GPT-4</option>
					</select>
				</div>

				<div>
					<label class="block text-sm font-semibold text-gray-900">Agent Prompt</label>
					<textarea
						bind:value={agentPrompt}
						placeholder="Optional custom instructions for the agent"
						rows={3}
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

				<div class="flex gap-3">
					<button
						onclick={() => {
							step = 2;
						}}
						class="flex-1 rounded-md border border-gray-300 px-4 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50"
					>
						Back
					</button>
					<button
						onclick={creationType === 'clone' ? handleCloneProject : handleCreateProject}
						disabled={loading}
						class="flex-1 rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:opacity-50"
					>
						{#if loading}
							Creating...
						{:else}
							Create Project
						{/if}
					</button>
				</div>
			</div>
		{/if}
	</main>
</div>
