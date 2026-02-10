<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	let projectId: number;
	let workflows = $state<any[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		projectId = parseInt($page.params.id, 10);
		if (isNaN(projectId)) {
			error = 'Invalid project ID';
			loading = false;
			return;
		}

		try {
			const response = await fetch(`/api/projects/${projectId}/workflows`);
			if (!response.ok) {
				throw new Error('Failed to load workflows');
			}
			workflows = await response.json();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load workflows';
		} finally {
			loading = false;
		}
	});

	function createWorkflow() {
		goto(`/projects/${projectId}/workflows/new`);
	}

	function editWorkflow(id: number) {
		goto(`/projects/${projectId}/workflows/${id}`);
	}

	function goBack() {
		goto(`/projects/${projectId}`);
	}
</script>

<svelte:head>
	<title>Workflows - ClickUp Orchestrator</title>
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<!-- Header -->
	<div class="bg-white shadow">
		<div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
			<button onclick={goBack} class="mb-4 inline-flex items-center text-sm text-indigo-600 hover:text-indigo-500">
				<svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
				</svg>
				Back to Project
			</button>
			<div class="flex items-center justify-between">
				<h1 class="text-3xl font-bold tracking-tight text-gray-900">Workflows</h1>
				<button
					onclick={createWorkflow}
					class="inline-flex items-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500"
				>
					<svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
					</svg>
					New Workflow
				</button>
			</div>
		</div>
	</div>

	<!-- Content -->
	<main class="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
		{#if loading}
			<div class="flex items-center justify-center py-12">
				<svg class="h-12 w-12 animate-spin text-indigo-600" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
			</div>
		{:else if error}
			<div class="rounded-md bg-red-50 p-4">
				<p class="text-sm text-red-700">{error}</p>
			</div>
		{:else if workflows.length === 0}
			<div class="text-center">
				<svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
				</svg>
				<h3 class="mt-2 text-sm font-semibold text-gray-900">No workflows</h3>
				<p class="mt-1 text-sm text-gray-500">Get started by creating your first workflow.</p>
				<button
					onclick={createWorkflow}
					class="mt-4 inline-flex items-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500"
				>
					Create workflow
				</button>
			</div>
		{:else}
			<div class="space-y-4">
				{#each workflows as workflow (workflow.id)}
					<button
						onclick={() => editWorkflow(workflow.id)}
						class="block w-full rounded-lg border border-gray-200 bg-white p-6 text-left shadow-sm transition-all hover:border-indigo-500 hover:shadow-md"
					>
						<div class="flex items-start justify-between">
							<div>
								<h3 class="text-lg font-semibold text-gray-900">{workflow.name}</h3>
								<p class="mt-1 text-sm text-gray-500">ID: {workflow.id}</p>
							</div>
							<span
								class="rounded-full px-3 py-1 text-xs font-semibold {workflow.status === 'running'
									? 'bg-green-100 text-green-700'
									: 'bg-gray-100 text-gray-700'}"
							>
								{workflow.status}
							</span>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</main>
</div>
