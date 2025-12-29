<script lang="ts">
	import { saveApiKey, completeSetup, type SetupStatus } from '$lib/api/setup';
	import { updateSettings } from '$lib/api/settings';
	import ClickUpBrowser from '$lib/components/settings/ClickUpBrowser.svelte';
	import GitConfig from '$lib/components/settings/GitConfig.svelte';

	interface Props {
		status: SetupStatus;
		onComplete: () => void;
	}

	let { status, onComplete }: Props = $props();

	// Determine initial step based on status
	function getInitialStep(): number {
		if (!status.has_api_key || !status.api_key_valid) return 1;
		if (!status.has_list_selected) return 2;
		if (!status.has_repo_configured) return 3;
		return 3;
	}

	let currentStep = $state(getInitialStep());
	let loading = $state(false);
	let error = $state<string | null>(null);

	// Step 1: API Key
	let apiKey = $state('');

	// Step 2: ClickUp List
	let workspaceId = $state('');
	let spaceId = $state('');
	let folderId = $state('');
	let listId = $state('');

	// Step 3: Git Config
	let repoPath = $state('');
	let branch = $state('');

	async function handleSaveApiKey() {
		if (!apiKey.trim()) {
			error = 'Please enter an API key';
			return;
		}

		loading = true;
		error = null;

		try {
			const result = await saveApiKey(apiKey.trim());
			if (result.success) {
				currentStep = 2;
			} else {
				error = result.error ?? 'Failed to save API key';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save API key';
		} finally {
			loading = false;
		}
	}

	function handleClickUpChange(selection: {
		workspaceId: string;
		spaceId: string;
		folderId: string;
		listId: string;
	}) {
		workspaceId = selection.workspaceId;
		spaceId = selection.spaceId;
		folderId = selection.folderId;
		listId = selection.listId;
	}

	async function handleSaveList() {
		if (!listId) {
			error = 'Please select a ClickUp list';
			return;
		}

		loading = true;
		error = null;

		try {
			await updateSettings({
				clickup_workspace_id: workspaceId,
				clickup_space_id: spaceId,
				clickup_folder_id: folderId,
				clickup_list_id: listId
			});
			currentStep = 3;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save list selection';
		} finally {
			loading = false;
		}
	}

	async function handleSaveGitConfig() {
		loading = true;
		error = null;

		try {
			// Save git config (optional, can be empty)
			await updateSettings({
				target_repo_path: repoPath,
				dev_branch: branch || 'dev'
			});

			// Complete setup
			const result = await completeSetup();
			if (result.success) {
				onComplete();
			} else {
				error = result.error ?? 'Failed to complete setup';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to complete setup';
		} finally {
			loading = false;
		}
	}

	function handleSkipGitConfig() {
		handleSaveGitConfig();
	}
</script>

<div class="min-h-screen bg-gradient-to-br from-indigo-100 via-white to-purple-100 flex items-center justify-center p-4">
	<div class="w-full max-w-2xl">
		<!-- Header -->
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold text-gray-900">Welcome to ClickUp Orchestrator</h1>
			<p class="mt-2 text-gray-600">Let's get you set up in a few quick steps</p>
		</div>

		<!-- Progress Steps -->
		<div class="mb-8">
			<div class="flex items-center justify-center gap-4">
				{#each [1, 2, 3] as step}
					<div class="flex items-center">
						<div
							class="flex items-center justify-center w-10 h-10 rounded-full text-sm font-medium
								{currentStep > step
									? 'bg-indigo-600 text-white'
									: currentStep === step
										? 'bg-indigo-600 text-white ring-4 ring-indigo-100'
										: 'bg-gray-200 text-gray-500'}"
						>
							{#if currentStep > step}
								<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
									<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
								</svg>
							{:else}
								{step}
							{/if}
						</div>
						{#if step < 3}
							<div class="w-16 h-1 mx-2 {currentStep > step ? 'bg-indigo-600' : 'bg-gray-200'}"></div>
						{/if}
					</div>
				{/each}
			</div>
			<div class="flex justify-center gap-4 mt-2 text-xs text-gray-500">
				<span class="w-24 text-center">API Key</span>
				<span class="w-24 text-center">Select List</span>
				<span class="w-24 text-center">Git Setup</span>
			</div>
		</div>

		<!-- Card -->
		<div class="bg-white rounded-2xl shadow-xl p-8">
			{#if error}
				<div class="mb-6 rounded-md bg-red-50 p-4">
					<div class="flex">
						<svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
						</svg>
						<p class="ml-3 text-sm text-red-700">{error}</p>
					</div>
				</div>
			{/if}

			<!-- Step 1: API Key -->
			{#if currentStep === 1}
				<div class="space-y-6">
					<div>
						<h2 class="text-xl font-semibold text-gray-900">Connect to ClickUp</h2>
						<p class="mt-1 text-sm text-gray-500">
							Enter your ClickUp API key to get started. You can find this in ClickUp Settings → Apps.
						</p>
					</div>

					<div>
						<label for="api-key" class="block text-sm font-medium text-gray-700">
							ClickUp API Key
						</label>
						<input
							type="password"
							id="api-key"
							bind:value={apiKey}
							placeholder="pk_..."
							class="mt-1 block w-full rounded-lg border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
						/>
						<p class="mt-2 text-xs text-gray-500">
							Your API key will be saved securely in a .env file.
						</p>
					</div>

					<div class="flex justify-end">
						<button
							onclick={handleSaveApiKey}
							disabled={loading || !apiKey.trim()}
							class="inline-flex items-center rounded-lg bg-indigo-600 px-6 py-3 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed"
						>
							{#if loading}
								<svg class="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								Validating...
							{:else}
								Continue
								<svg class="ml-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
								</svg>
							{/if}
						</button>
					</div>
				</div>
			{/if}

			<!-- Step 2: Select List -->
			{#if currentStep === 2}
				<div class="space-y-6">
					<div>
						<h2 class="text-xl font-semibold text-gray-900">Select a ClickUp List</h2>
						<p class="mt-1 text-sm text-gray-500">
							Choose the list that the orchestrator will monitor for tasks.
						</p>
					</div>

					<ClickUpBrowser
						{workspaceId}
						{spaceId}
						{folderId}
						{listId}
						onchange={handleClickUpChange}
					/>

					<div class="flex justify-between">
						<button
							onclick={() => (currentStep = 1)}
							class="inline-flex items-center rounded-lg bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
						>
							<svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
							</svg>
							Back
						</button>
						<button
							onclick={handleSaveList}
							disabled={loading || !listId}
							class="inline-flex items-center rounded-lg bg-indigo-600 px-6 py-3 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed"
						>
							{#if loading}
								<svg class="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								Saving...
							{:else}
								Continue
								<svg class="ml-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
								</svg>
							{/if}
						</button>
					</div>
				</div>
			{/if}

			<!-- Step 3: Git Config -->
			{#if currentStep === 3}
				<div class="space-y-6">
					<div>
						<h2 class="text-xl font-semibold text-gray-900">Configure Git Repository</h2>
						<p class="mt-1 text-sm text-gray-500">
							Set up the git repository where tasks will be worked on. You can skip this and configure it later.
						</p>
					</div>

					<GitConfig
						{repoPath}
						{branch}
						onPathChange={(p) => (repoPath = p)}
						onBranchChange={(b) => (branch = b)}
					/>

					<div class="flex justify-between">
						<button
							onclick={() => (currentStep = 2)}
							class="inline-flex items-center rounded-lg bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
						>
							<svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
							</svg>
							Back
						</button>
						<div class="flex gap-3">
							<button
								onclick={handleSkipGitConfig}
								disabled={loading}
								class="inline-flex items-center rounded-lg bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 disabled:opacity-50"
							>
								Skip for now
							</button>
							<button
								onclick={handleSaveGitConfig}
								disabled={loading}
								class="inline-flex items-center rounded-lg bg-indigo-600 px-6 py-3 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed"
							>
								{#if loading}
									<svg class="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24">
										<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
										<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
									</svg>
									Finishing...
								{:else}
									Finish Setup
									<svg class="ml-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
									</svg>
								{/if}
							</button>
						</div>
					</div>
				</div>
			{/if}
		</div>

		<!-- Help text -->
		<p class="mt-6 text-center text-xs text-gray-500">
			Need help? Check out the <a href="https://clickup.com/api" target="_blank" rel="noopener" class="text-indigo-600 hover:text-indigo-500">ClickUp API documentation</a>
		</p>
	</div>
</div>
