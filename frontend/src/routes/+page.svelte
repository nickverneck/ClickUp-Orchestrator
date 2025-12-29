<script lang="ts">
	import { onMount } from 'svelte';
	import KanbanBoard from '$lib/components/kanban/KanbanBoard.svelte';
	import SetupWizard from '$lib/components/setup/SetupWizard.svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import { getSetupStatus, type SetupStatus } from '$lib/api/setup';

	let setupStatus = $state<SetupStatus | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let sidebarCollapsed = $state(false);

	onMount(() => {
		checkSetup();
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

	async function checkSetup() {
		loading = true;
		error = null;
		try {
			setupStatus = await getSetupStatus();
		} catch (e) {
			// If we can't reach the backend, show a connection error
			error = e instanceof Error ? e.message : 'Failed to connect to backend';
		} finally {
			loading = false;
		}
	}

	function handleSetupComplete() {
		// Reload setup status
		checkSetup();
	}
</script>

<svelte:head>
	<title>ClickUp Orchestrator</title>
</svelte:head>

{#if loading}
	<!-- Loading state -->
	<div class="min-h-screen bg-gray-50 flex items-center justify-center">
		<div class="text-center">
			<svg class="mx-auto h-12 w-12 animate-spin text-indigo-600" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
			</svg>
			<p class="mt-4 text-gray-500">Loading...</p>
		</div>
	</div>
{:else if error}
	<!-- Connection error -->
	<div class="min-h-screen bg-gray-50 flex items-center justify-center">
		<div class="text-center max-w-md">
			<svg class="mx-auto h-12 w-12 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
			</svg>
			<h2 class="mt-4 text-lg font-semibold text-gray-900">Connection Error</h2>
			<p class="mt-2 text-sm text-gray-500">{error}</p>
			<p class="mt-2 text-sm text-gray-500">Make sure the backend is running on port 5150.</p>
			<button
				onclick={checkSetup}
				class="mt-4 inline-flex items-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500"
			>
				Retry
			</button>
		</div>
	</div>
{:else if setupStatus && !setupStatus.is_complete}
	<!-- Setup Wizard -->
	<SetupWizard status={setupStatus} onComplete={handleSetupComplete} />
{:else}
	<!-- Main Dashboard with Sidebar -->
	<div class="flex h-screen bg-gray-50">
		<Sidebar bind:collapsed={sidebarCollapsed} />

		<main class="flex-1 overflow-auto">
			<div class="p-8">
				<KanbanBoard />
			</div>
		</main>
	</div>
{/if}
