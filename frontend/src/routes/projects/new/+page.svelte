<script lang="ts">
	import { createProject, cloneProjectFromGithub, type CreateProjectRequest, checkGitStatus, initializeGit } from '$lib/api/projects';
	import FolderPicker from '$lib/components/projects/FolderPicker.svelte';
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

	// Git status
	let gitStatus = $state<{ isGitRepo: boolean; branch?: string; loading: boolean; error?: string } | null>(null);

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

	async function handleFolderSelect(path: string, isNewRepo: boolean) {
		if (isNewRepo) {
			repoPath = path;
			await checkRepoGitStatus(path);
		} else {
			targetPath = path;
			await checkRepoGitStatus(path);
		}
	}

	async function checkRepoGitStatus(path: string) {
		gitStatus = { isGitRepo: false, loading: true };
		try {
			const status = await checkGitStatus(path);
			gitStatus = {
				isGitRepo: status.is_git_repo,
				branch: status.branch || 'main',
				loading: false,
			};
			if (status.is_git_repo && status.branch) {
				devBranch = status.branch;
			}
		} catch (err) {
			gitStatus = {
				isGitRepo: false,
				loading: false,
				error: err instanceof Error ? err.message : 'Failed to check git status',
			};
		}
	}

	async function handleInitializeGit(path: string) {
		if (!gitStatus) return;
		gitStatus.loading = true;
		gitStatus.error = undefined;

		try {
			const result = await initializeGit(path);
			gitStatus = {
				isGitRepo: true,
				branch: result.branch,
				loading: false,
			};
			devBranch = result.branch;
		} catch (err) {
			gitStatus = {
				isGitRepo: false,
				loading: false,
				error: err instanceof Error ? err.message : 'Failed to initialize git',
			};
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
					<label for="project-name" class="block text-sm font-semibold text-gray-900">Project Name *</label>
					<input
						id="project-name"
						type="text"
						bind:value={name}
						placeholder="My Project"
						class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
					/>
				</div>

				<div>
					<label for="description" class="block text-sm font-semibold text-gray-900">Description</label>
					<textarea
						id="description"
						bind:value={description}
						placeholder="Optional project description"
						rows={3}
						class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
					></textarea>
				</div>

				{#if creationType === 'new'}
					<div>
						<div class="block text-sm font-semibold text-gray-900">Repository Path *</div>
						<p class="mt-1 text-xs text-gray-500">Select an existing folder (will initialize git if needed)</p>
						<div class="mt-3">
							<FolderPicker onSelect={(path) => handleFolderSelect(path, true)} selectedPath={repoPath} />
						</div>
					</div>

					{#if repoPath && gitStatus}
						{#if gitStatus.loading}
							<div class="rounded-md bg-blue-50 p-3">
								<p class="text-sm text-blue-700">Checking git status...</p>
							</div>
						{:else if gitStatus.error}
							<div class="rounded-md bg-red-50 p-3">
								<p class="text-sm text-red-700">{gitStatus.error}</p>
							</div>
						{:else if gitStatus.isGitRepo}
							<div class="rounded-md bg-green-50 p-3">
								<p class="text-sm text-green-700">✓ Git repository found (branch: {gitStatus.branch})</p>
							</div>
						{:else}
							<div class="space-y-3 rounded-md bg-amber-50 p-3">
								<p class="text-sm text-amber-700">This folder is not a git repository. Would you like to initialize one?</p>
								<button
									onclick={() => handleInitializeGit(repoPath)}
									disabled={gitStatus.loading}
									class="inline-flex items-center rounded-md bg-amber-600 px-3 py-2 text-sm font-medium text-white hover:bg-amber-700 disabled:opacity-50"
								>
									{#if gitStatus.loading}
										Initializing...
									{:else}
										Initialize Git
									{/if}
								</button>
							</div>
						{/if}
					{/if}
				{:else if creationType === 'clone'}
					<div>
						<label for="github-url" class="block text-sm font-semibold text-gray-900">GitHub Repository URL *</label>
						<input
							id="github-url"
							type="text"
							bind:value={githubUrl}
							placeholder="https://github.com/user/repo"
							class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
						/>
					</div>

					<div>
						<div class="block text-sm font-semibold text-gray-900">Clone Target Path *</div>
						<p class="mt-1 text-xs text-gray-500">Select where to clone the repository</p>
						<div class="mt-3">
							<FolderPicker onSelect={(path) => handleFolderSelect(path, false)} selectedPath={targetPath} />
						</div>
					</div>
				{/if}

				<div>
					<label for="dev-branch" class="block text-sm font-semibold text-gray-900">Development Branch</label>
					<input
						id="dev-branch"
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
					<label for="clickup-api-key" class="block text-sm font-semibold text-gray-900">
						ClickUp API Key
						<span class="font-normal text-gray-500">(optional)</span>
					</label>
					<input
						id="clickup-api-key"
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
						<label for="clickup-list-id" class="block text-sm font-semibold text-gray-900">ClickUp List ID</label>
						<input
							id="clickup-list-id"
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
					<label for="agent-model" class="block text-sm font-semibold text-gray-900">Agent Model</label>
					<select
						id="agent-model"
						bind:value={agentModel}
						class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
					>
						<option value="claude">Claude</option>
						<option value="gpt4">GPT-4</option>
					</select>
				</div>

				<div>
					<label for="agent-prompt" class="block text-sm font-semibold text-gray-900">Agent Prompt</label>
					<textarea
						id="agent-prompt"
						bind:value={agentPrompt}
						placeholder="Optional custom instructions for the agent"
						rows={3}
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
