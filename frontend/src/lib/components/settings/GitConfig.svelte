<script lang="ts">
	import { validatePath, getBranches, fetchRepo, detectPath } from '$lib/api/git';

	interface Props {
		repoPath: string;
		branch: string;
		onPathChange: (path: string) => void;
		onBranchChange: (branch: string) => void;
	}

	let { repoPath, branch, onPathChange, onBranchChange }: Props = $props();

	let branches = $state<string[]>([]);
	let currentBranch = $state<string | undefined>(undefined);
	let isValid = $state<boolean | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let fetchingBranches = $state(false);
	let browseSupported = $state(false);
	let browsing = $state(false);

	// Check if File System Access API is supported
	$effect(() => {
		browseSupported = 'showDirectoryPicker' in window;
	});

	// Validate path when it changes (debounced)
	let validateTimeout: ReturnType<typeof setTimeout>;
	$effect(() => {
		if (repoPath) {
			clearTimeout(validateTimeout);
			validateTimeout = setTimeout(() => validateRepoPath(repoPath), 500);
		} else {
			isValid = null;
			branches = [];
		}
	});

	async function handleBrowse() {
		try {
			browsing = true;
			error = null;

			// @ts-expect-error - showDirectoryPicker is not in all TS libs
			const dirHandle = await window.showDirectoryPicker({
				mode: 'readwrite' // Need write permission to create marker file
			});

			// Generate a unique marker filename
			const markerId = crypto.randomUUID();
			const markerFilename = `.clickup-orchestrator-path-${markerId}`;

			// Create the marker file in the selected directory
			try {
				const fileHandle = await dirHandle.getFileHandle(markerFilename, { create: true });
				const writable = await fileHandle.createWritable();
				await writable.write('This file is used for path detection. You can delete it.');
				await writable.close();
			} catch (e) {
				error = 'Could not create marker file. Please enter the path manually.';
				browsing = false;
				return;
			}

			// Ask the backend to find the marker file
			try {
				const result = await detectPath(markerFilename);

				// Clean up - delete the marker file
				try {
					await dirHandle.removeEntry(markerFilename);
				} catch {
					// Ignore cleanup errors
				}

				if (result.found && result.path) {
					onPathChange(result.path);
				} else {
					// Fallback to manual entry if not found
					const guessedPath = prompt(
						`Could not auto-detect path for "${dirHandle.name}".\n\nPlease enter the full path to this folder:`,
						repoPath || `/path/to/${dirHandle.name}`
					);
					if (guessedPath) {
						onPathChange(guessedPath);
					}
				}
			} catch (e) {
				error = 'Path detection failed. Please enter the path manually.';
				// Try to clean up marker file
				try {
					await dirHandle.removeEntry(markerFilename);
				} catch {
					// Ignore cleanup errors
				}
			}
		} catch (e) {
			// User cancelled or API not supported
			if (e instanceof Error && e.name !== 'AbortError') {
				console.error('Browse failed:', e);
				error = 'Could not open folder picker. Please enter the path manually.';
			}
		} finally {
			browsing = false;
		}
	}

	async function validateRepoPath(path: string) {
		loading = true;
		error = null;
		try {
			const result = await validatePath(path);
			isValid = result.valid;
			if (result.valid) {
				await loadBranches(path);
			} else {
				error = result.error ?? 'Invalid repository';
				branches = [];
			}
		} catch (e) {
			isValid = false;
			error = e instanceof Error ? e.message : 'Validation failed';
			branches = [];
		} finally {
			loading = false;
		}
	}

	async function loadBranches(path: string) {
		fetchingBranches = true;
		try {
			const result = await getBranches(path);
			branches = result.branches;
			currentBranch = result.current;
			// Auto-select current branch if no branch is set
			if (!branch && result.current) {
				onBranchChange(result.current);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load branches';
			branches = [];
		} finally {
			fetchingBranches = false;
		}
	}

	async function handleFetch() {
		if (!repoPath || !isValid) return;
		loading = true;
		error = null;
		try {
			await fetchRepo(repoPath);
			await loadBranches(repoPath);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Fetch failed';
		} finally {
			loading = false;
		}
	}

	function handlePathInput(e: Event) {
		onPathChange((e.target as HTMLInputElement).value);
	}

	function handleBranchChange(e: Event) {
		onBranchChange((e.target as HTMLSelectElement).value);
	}
</script>

<div class="space-y-4">
	<h3 class="text-lg font-medium text-gray-900">Git Repository Configuration</h3>

	{#if error}
		<div class="rounded-md bg-red-50 p-4">
			<p class="text-sm text-red-700">{error}</p>
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
		<!-- Repository Path -->
		<div class="lg:col-span-2">
			<label for="repo-path" class="block text-sm font-medium text-gray-700">Repository Path</label>
			<div class="mt-1 flex gap-2">
				{#if browseSupported}
					<button
						type="button"
						onclick={handleBrowse}
						disabled={browsing}
						class="inline-flex items-center rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
						title="Browse for folder"
					>
						{#if browsing}
							<svg class="h-5 w-5 animate-spin text-gray-400" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
						{:else}
							<svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
							</svg>
						{/if}
					</button>
				{/if}
				<div class="relative flex-1">
					<input
						type="text"
						id="repo-path"
						value={repoPath}
						oninput={handlePathInput}
						placeholder="/path/to/your/repository"
						class="block w-full rounded-md border-gray-300 pr-10 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
						class:border-green-300={isValid === true}
						class:border-red-300={isValid === false}
					/>
					{#if isValid !== null && !loading}
						<div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
							{#if isValid}
								<svg class="h-5 w-5 text-green-500" viewBox="0 0 20 20" fill="currentColor">
									<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
								</svg>
							{:else}
								<svg class="h-5 w-5 text-red-500" viewBox="0 0 20 20" fill="currentColor">
									<path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
								</svg>
							{/if}
						</div>
					{/if}
				</div>
				<button
					type="button"
					onclick={handleFetch}
					disabled={!isValid || loading}
					class="inline-flex items-center rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{#if loading}
						<svg class="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
					{/if}
					Fetch
				</button>
			</div>
			<p class="mt-1 text-sm text-gray-500">
				{#if browseSupported}
					Click the folder icon to browse, or enter the absolute path manually
				{:else}
					Enter the absolute path to your git repository
				{/if}
			</p>
		</div>

		<!-- Branch Selector -->
		<div>
			<label for="branch" class="block text-sm font-medium text-gray-700">Development Branch</label>
			<select
				id="branch"
				value={branch}
				onchange={handleBranchChange}
				disabled={branches.length === 0 || fetchingBranches}
				class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
			>
				<option value="">Select branch...</option>
				{#each branches as b}
					<option value={b}>
						{b}{b === currentBranch ? ' (current)' : ''}
					</option>
				{/each}
			</select>
			<p class="mt-1 text-sm text-gray-500">
				Branch to base worktrees from
			</p>
		</div>

		<!-- Current Branch Info -->
		{#if currentBranch}
			<div class="flex items-center gap-2">
				<span class="text-sm text-gray-500">Current branch:</span>
				<span class="inline-flex items-center rounded-full bg-indigo-100 px-2.5 py-0.5 text-xs font-medium text-indigo-800">
					{currentBranch}
				</span>
			</div>
		{/if}
	</div>
</div>
