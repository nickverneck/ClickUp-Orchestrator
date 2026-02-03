<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import {
		createWorkflow,
		deleteWorkflow,
		getWorkflows,
		pauseWorkflow,
		startWorkflow,
		type WorkflowSummary
	} from '$lib/api/workflow';

	let workflows = $state<WorkflowSummary[]>([]);
	let loading = $state(true);
	let creating = $state(false);
	let error = $state<string | null>(null);
	let newName = $state('');
	let actionId = $state<number | null>(null);
	let deletingId = $state<number | null>(null);
	let sidebarCollapsed = $state(false);

	onMount(() => {
		loadWorkflows();
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}
	});

	$effect(() => {
		localStorage.setItem('sidebarCollapsed', String(sidebarCollapsed));
	});

	async function loadWorkflows() {
		loading = true;
		error = null;
		try {
			workflows = await getWorkflows();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load workflows';
		} finally {
			loading = false;
		}
	}

	async function handleCreate() {
		if (creating) return;
		creating = true;
		error = null;
		try {
			const name = newName.trim();
			const created = await createWorkflow(name.length > 0 ? name : undefined);
			newName = '';
			goto(`/workflow/${created.id}`);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create workflow';
		} finally {
			creating = false;
		}
	}

	async function toggleStatus(item: WorkflowSummary) {
		if (actionId) return;
		actionId = item.id;
		error = null;
		try {
			const response =
				item.status === 'running'
					? await pauseWorkflow(item.id)
					: await startWorkflow(item.id);
			workflows = workflows.map((workflow) =>
				workflow.id === item.id
					? { ...workflow, status: response.status, name: response.name || workflow.name }
					: workflow
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update workflow status';
		} finally {
			actionId = null;
		}
	}

	async function handleDelete(id: number) {
		if (deletingId) return;
		const confirmed = window.confirm('Delete this workflow? This cannot be undone.');
		if (!confirmed) return;
		deletingId = id;
		error = null;
		try {
			await deleteWorkflow(id);
			workflows = workflows.filter((workflow) => workflow.id !== id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete workflow';
		} finally {
			deletingId = null;
		}
	}
</script>

<svelte:head>
	<title>Workflows - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex h-screen bg-gray-50">
	<Sidebar bind:collapsed={sidebarCollapsed} />

	<main class="flex-1 overflow-auto">
		<div class="mx-auto max-w-5xl px-6 py-8">
			<div class="flex flex-wrap items-center justify-between gap-4">
				<div>
					<h1 class="text-2xl font-semibold text-gray-900">Workflows</h1>
					<p class="text-sm text-gray-500">
						Create and run multiple pipelines in parallel.
					</p>
				</div>
			</div>

			<div class="mt-6 rounded-lg bg-white p-5 shadow">
				<div class="flex flex-col gap-3 sm:flex-row sm:items-end">
					<div class="flex-1">
						<label for="workflow-name" class="block text-sm font-medium text-gray-700">
							Workflow Name
						</label>
						<input
							id="workflow-name"
							class="mt-1 w-full rounded-md border border-gray-300 text-sm shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
							placeholder="e.g. Marketing Intake"
							value={newName}
							oninput={(event) => (newName = event.currentTarget.value)}
							disabled={creating}
						/>
					</div>
					<button
						onclick={handleCreate}
						disabled={creating}
						class="inline-flex items-center justify-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-50"
					>
						{creating ? 'Creating...' : 'Create Workflow'}
					</button>
				</div>
				<p class="mt-2 text-sm text-gray-500">
					New workflows start paused. Edit the pipeline, then run it when ready.
				</p>
			</div>

			{#if error}
				<div class="mt-6 rounded-md bg-red-50 p-4 text-sm text-red-700">{error}</div>
			{/if}

			{#if loading}
				<div class="mt-10 flex items-center justify-center">
					<svg class="h-8 w-8 animate-spin text-indigo-600" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
				</div>
			{:else if workflows.length === 0}
				<div class="mt-10 rounded-lg border border-dashed border-gray-300 bg-white p-10 text-center">
					<h3 class="text-lg font-semibold text-gray-900">No workflows yet</h3>
					<p class="mt-2 text-sm text-gray-500">
						Create your first workflow to start orchestrating tasks.
					</p>
				</div>
			{:else}
				<div class="mt-6 grid gap-4">
					{#each workflows as workflow}
						<div class="rounded-lg bg-white p-4 shadow-sm">
							<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
								<div>
									<div class="flex items-center gap-3">
										<h3 class="text-lg font-semibold text-gray-900">{workflow.name}</h3>
										<span
											class="inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-xs font-semibold uppercase tracking-wide {workflow.status === 'running'
												? 'bg-emerald-100 text-emerald-700'
												: 'bg-amber-100 text-amber-700'}"
										>
											<span
												class="h-2 w-2 rounded-full {workflow.status === 'running'
													? 'bg-emerald-500'
													: 'bg-amber-500'}"
											></span>
											{workflow.status}
										</span>
									</div>
									<p class="mt-1 text-sm text-gray-500">Workflow ID {workflow.id}</p>
								</div>
								<div class="flex flex-wrap gap-2">
									<button
										onclick={() => goto(`/workflow/${workflow.id}`)}
										class="inline-flex items-center rounded-md border border-gray-200 px-3 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50"
									>
										Edit
									</button>
									<button
										onclick={() => toggleStatus(workflow)}
										disabled={actionId === workflow.id}
										class="inline-flex items-center rounded-md border border-gray-200 px-3 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
									>
										{workflow.status === 'running' ? 'Pause' : 'Run'}
									</button>
									<button
										onclick={() => handleDelete(workflow.id)}
										disabled={deletingId === workflow.id}
										class="inline-flex items-center rounded-md border border-red-200 px-3 py-2 text-sm font-semibold text-red-600 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-50"
									>
										Delete
									</button>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</main>
</div>
