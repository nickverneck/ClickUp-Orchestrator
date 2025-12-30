<script lang="ts">
	import { post } from '$lib/api/client';
	import { getSettings } from '$lib/api/settings';
	import { onMount } from 'svelte';

	interface Props {
		onBranchCreate: (branchName: string) => void;
	}

	let { onBranchCreate }: Props = $props();

	let isCreating = $state(false);
	let branchName = $state('');
	let error = $state('');
	let targetRepoPath = $state('');

	// Auto-generate branch name suggestion
	const suggestedName = `ui-refinements-${new Date().toISOString().slice(0, 10)}-${Math.random().toString(36).slice(2, 6)}`;

	onMount(async () => {
		branchName = suggestedName;
		try {
			const settings = await getSettings();
			if (settings.target_repo_path) {
				targetRepoPath = settings.target_repo_path;
			}
		} catch (e) {
			console.error('Failed to load target_repo_path:', e);
		}
	});

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
			<h2 class="text-xl font-semibold text-white">Create a Branch</h2>
			<p class="text-indigo-200 text-sm mt-1">
				Protect your main branch while prototyping UI changes
			</p>
		</div>

		<!-- Content -->
		<div class="px-6 py-5">
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
				<div>
					<label for="branchName" class="block text-sm font-medium text-gray-700 mb-1">
						Branch Name
					</label>
					<input
						id="branchName"
						type="text"
						bind:value={branchName}
						placeholder="feature/ui-updates"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent font-mono text-sm"
						disabled={isCreating}
					/>
				</div>

				{#if targetRepoPath}
					<div class="text-sm text-gray-500">
						<span class="font-medium">Repository:</span>
						<span class="font-mono ml-1">{targetRepoPath}</span>
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
			<button
				onclick={handleCreate}
				disabled={isCreating || !branchName.trim()}
				class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-600 text-white font-medium rounded-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
			>
				{#if isCreating}
					<svg class="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
					<span>Creating Branch...</span>
				{:else}
					<svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M13 10V3L4 14h7v7l9-11h-7z"
						/>
					</svg>
					<span>Create Branch & Start</span>
				{/if}
			</button>
		</div>
	</div>
</div>
