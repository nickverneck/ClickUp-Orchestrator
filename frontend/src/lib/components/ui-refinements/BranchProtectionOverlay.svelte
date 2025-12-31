<script lang="ts">
	import { get, post } from '$lib/api/client';
	import { getSettings } from '$lib/api/settings';
	import { onMount } from 'svelte';

	interface Props {
		onBranchCreate: (branchName: string) => void;
	}

	let { onBranchCreate }: Props = $props();

	let isLoading = $state(true);
	let isCreating = $state(false);
	let branchName = $state('');
	let error = $state('');
	let targetRepoPath = $state('');
	let existingBranches = $state<string[]>([]);
	let selectedExisting = $state<string | null>(null);
	let mode = $state<'select' | 'create'>('select');

	// Auto-generate branch name suggestion
	const suggestedName = `ui-refinements-${new Date().toISOString().slice(0, 10)}-${Math.random().toString(36).slice(2, 6)}`;

	onMount(async () => {
		branchName = suggestedName;
		try {
			const settings = await getSettings();
			if (settings.target_repo_path) {
				targetRepoPath = settings.target_repo_path;
				await loadExistingBranches();
			}
		} catch (e) {
			console.error('Failed to load settings:', e);
		} finally {
			isLoading = false;
		}
	});

	async function loadExistingBranches() {
		try {
			const response = await get<{ branches: string[]; current: string | null }>(
				`/git/branches?path=${encodeURIComponent(targetRepoPath)}`
			);
			existingBranches = response.branches.filter((b) => b.startsWith('ui-refinements'));
			if (existingBranches.length === 0) {
				mode = 'create';
			}
		} catch (e) {
			console.error('Failed to load branches:', e);
			mode = 'create';
		}
	}

	async function handleSelectExisting() {
		if (!selectedExisting) {
			error = 'Please select a branch';
			return;
		}

		isCreating = true;
		error = '';

		try {
			const response = await post<{ success: boolean; error?: string }>('/git/checkout', {
				path: targetRepoPath,
				branch_name: selectedExisting
			});

			if (response.success) {
				onBranchCreate(selectedExisting);
			} else {
				error = response.error || 'Failed to checkout branch';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to checkout branch';
		} finally {
			isCreating = false;
		}
	}

	async function handleCreate() {
		if (!branchName.trim()) {
			error = 'Branch name is required';
			return;
		}

		if (!targetRepoPath) {
			error = 'No target repository configured. Please set one in Settings.';
			return;
		}

		isCreating = true;
		error = '';

		try {
			const response = await post<{ success: boolean; error?: string }>('/git/create-branch', {
				path: targetRepoPath,
				branch_name: branchName.trim()
			});

			if (response.success) {
				onBranchCreate(branchName.trim());
			} else {
				error = response.error || 'Failed to create branch';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create branch';
		} finally {
			isCreating = false;
		}
	}
</script>

<div class="absolute inset-0 z-50 flex items-center justify-center">
	<!-- Backdrop -->
	<div class="absolute inset-0 bg-gray-900/60 backdrop-blur-sm"></div>

	<!-- Modal -->
	<div class="relative bg-white rounded-xl shadow-2xl max-w-md w-full mx-4 overflow-hidden">
		<!-- Header -->
		<div class="bg-gradient-to-r from-indigo-600 to-indigo-700 px-6 py-4">
			<h2 class="text-xl font-semibold text-white">Select or Create Branch</h2>
			<p class="text-indigo-200 text-sm mt-1">
				Protect your main branch while prototyping UI changes
			</p>
		</div>

		{#if isLoading}
			<div class="px-6 py-12 flex items-center justify-center">
				<svg class="animate-spin h-8 w-8 text-indigo-600" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
				</svg>
			</div>
		{:else}
			<!-- Content -->
			<div class="px-6 py-5">
				<!-- Mode Tabs -->
				{#if existingBranches.length > 0}
					<div class="flex gap-2 mb-4">
						<button
							onclick={() => (mode = 'select')}
							class="flex-1 py-2 px-3 text-sm font-medium rounded-lg transition-colors {mode === 'select'
								? 'bg-indigo-100 text-indigo-700'
								: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
						>
							Continue Existing
						</button>
						<button
							onclick={() => (mode = 'create')}
							class="flex-1 py-2 px-3 text-sm font-medium rounded-lg transition-colors {mode === 'create'
								? 'bg-indigo-100 text-indigo-700'
								: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
						>
							Create New
						</button>
					</div>
				{/if}

				<div class="mb-4">
					<div class="flex items-start gap-3 p-3 bg-amber-50 rounded-lg border border-amber-200">
						<svg
							class="w-5 h-5 text-amber-600 flex-shrink-0 mt-0.5"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
							/>
						</svg>
						<p class="text-sm text-amber-800">
							All UI modifications will be made on a separate branch. You can merge changes back to
							main when ready.
						</p>
					</div>
				</div>

				<div class="space-y-4">
					{#if mode === 'select' && existingBranches.length > 0}
						<!-- Select Existing Branch -->
						<div>
							<label class="block text-sm font-medium text-gray-700 mb-2">
								Select a branch to continue
							</label>
							<div class="space-y-2 max-h-48 overflow-y-auto">
								{#each existingBranches as branch}
									<button
										onclick={() => (selectedExisting = branch)}
										class="w-full flex items-center gap-3 px-3 py-2 text-left rounded-lg border transition-colors {selectedExisting === branch
											? 'border-indigo-500 bg-indigo-50'
											: 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'}"
									>
										<svg
											class="w-4 h-4 flex-shrink-0 {selectedExisting === branch ? 'text-indigo-600' : 'text-gray-400'}"
											fill="none"
											viewBox="0 0 24 24"
											stroke="currentColor"
										>
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
										</svg>
										<span class="font-mono text-sm truncate {selectedExisting === branch ? 'text-indigo-700' : 'text-gray-700'}">
											{branch}
										</span>
										{#if selectedExisting === branch}
											<svg class="w-4 h-4 ml-auto text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
											</svg>
										{/if}
									</button>
								{/each}
							</div>
						</div>
					{:else}
						<!-- Create New Branch -->
						<div>
							<label for="branchName" class="block text-sm font-medium text-gray-700 mb-1">
								Branch Name
							</label>
							<input
								id="branchName"
								type="text"
								bind:value={branchName}
								placeholder="ui-refinements-..."
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent font-mono text-sm"
								disabled={isCreating}
							/>
						</div>
					{/if}

					{#if targetRepoPath}
						<div class="text-sm text-gray-500">
							<span class="font-medium">Repository:</span>
							<span class="font-mono ml-1 truncate">{targetRepoPath}</span>
						</div>
					{/if}

					{#if error}
						<div class="p-3 bg-red-50 text-red-700 text-sm rounded-lg border border-red-200">
							{error}
						</div>
					{/if}
				</div>
			</div>

			<!-- Footer -->
			<div class="px-6 py-4 bg-gray-50 border-t border-gray-200">
				{#if mode === 'select' && existingBranches.length > 0}
					<button
						onclick={handleSelectExisting}
						disabled={isCreating || !selectedExisting}
						class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-600 text-white font-medium rounded-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						{#if isCreating}
							<svg class="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
							</svg>
							<span>Switching Branch...</span>
						{:else}
							<svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
							</svg>
							<span>Continue with Branch</span>
						{/if}
					</button>
				{:else}
					<button
						onclick={handleCreate}
						disabled={isCreating || !branchName.trim()}
						class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-600 text-white font-medium rounded-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						{#if isCreating}
							<svg class="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
							</svg>
							<span>Creating Branch...</span>
						{:else}
							<svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
							</svg>
							<span>Create Branch & Start</span>
						{/if}
					</button>
				{/if}
			</div>
		{/if}
	</div>
</div>
