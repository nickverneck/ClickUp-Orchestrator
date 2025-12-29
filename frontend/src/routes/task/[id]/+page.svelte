<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import Terminal from '$lib/components/terminal/Terminal.svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import ConfirmModal from '$lib/components/ui/ConfirmModal.svelte';
	import { getTask, stopTask, restartTask, deleteTask, type Task } from '$lib/api/tasks';

	let task = $state<Task | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let sidebarCollapsed = $state(false);
	let deleteModalOpen = $state(false);

	const taskId = $derived(parseInt($page.params.id ?? '0', 10));

	onMount(() => {
		loadTask();
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

	async function loadTask() {
		loading = true;
		error = null;
		try {
			task = await getTask(taskId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load task';
		} finally {
			loading = false;
		}
	}

	async function handleStop() {
		if (!task) return;
		try {
			task = await stopTask(task.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to stop task';
		}
	}

	async function handleRestart() {
		if (!task) return;
		try {
			task = await restartTask(task.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to restart task';
		}
	}

	function handleKill() {
		// Reload task after kill
		setTimeout(loadTask, 500);
	}

	function handleDeleteClick() {
		deleteModalOpen = true;
	}

	async function confirmDelete() {
		if (!task) return;
		try {
			await deleteTask(task.id);
			goto('/');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete task';
		}
		deleteModalOpen = false;
	}

	function cancelDelete() {
		deleteModalOpen = false;
	}

	function formatDuration(ms: number): string {
		const seconds = Math.floor(ms / 1000);
		const minutes = Math.floor(seconds / 60);
		const hours = Math.floor(minutes / 60);

		if (hours > 0) {
			return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
		} else if (minutes > 0) {
			return `${minutes}m ${seconds % 60}s`;
		} else {
			return `${seconds}s`;
		}
	}

	function getPriorityLabel(priority?: number): string {
		switch (priority) {
			case 1:
				return 'Urgent';
			case 2:
				return 'High';
			case 3:
				return 'Normal';
			case 4:
				return 'Low';
			default:
				return 'None';
		}
	}
</script>

<svelte:head>
	<title>{task?.name ?? 'Task'} - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex h-screen bg-gray-100">
	<Sidebar bind:collapsed={sidebarCollapsed} />

	<div class="flex flex-1 flex-col overflow-hidden">
		<!-- Header -->
		<header class="bg-white shadow">
			<div class="px-4 py-4 sm:px-6 lg:px-8">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-4">
						<a
							href="/"
							class="inline-flex items-center text-sm text-gray-500 hover:text-gray-700"
						>
							<svg class="mr-1 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
							</svg>
							Back
						</a>
						{#if task}
							<h1 class="text-xl font-bold text-gray-900">{task.name}</h1>
							<span class="inline-flex items-center rounded-full bg-blue-100 px-2.5 py-0.5 text-xs font-medium text-blue-800">
								{task.status}
							</span>
						{/if}
					</div>

					{#if task}
						<div class="flex items-center gap-3">
							<span class="text-sm text-gray-500">
								Priority: <span class="font-medium">{getPriorityLabel(task.priority)}</span>
							</span>
							<span class="text-sm text-gray-500">
								Time: <span class="font-medium">{formatDuration(task.time_spent_ms)}</span>
							</span>

							{#if task.status === 'in_progress'}
								<button
									onclick={handleStop}
									class="inline-flex items-center rounded-md bg-red-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-red-500"
								>
									Stop
								</button>
							{/if}

							{#if task.status === 'stopped' || task.status === 'failed'}
								<button
									onclick={handleRestart}
									class="inline-flex items-center rounded-md bg-green-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-green-500"
								>
									Restart
								</button>
							{/if}

							{#if task.status === 'stopped' || task.status === 'failed' || task.status === 'completed'}
								<button
									onclick={handleDeleteClick}
									class="inline-flex items-center rounded-md bg-gray-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-red-600"
								>
									Delete
								</button>
							{/if}
						</div>
					{/if}
				</div>
			</div>
		</header>

		<!-- Main content -->
		<main class="flex-1 overflow-hidden p-4">
		{#if loading}
			<div class="flex h-full items-center justify-center">
				<svg class="h-8 w-8 animate-spin text-indigo-600" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
			</div>
		{:else if error}
			<div class="flex h-full items-center justify-center">
				<div class="rounded-md bg-red-50 p-4">
					<p class="text-sm text-red-700">{error}</p>
					<button
						onclick={() => goto('/')}
						class="mt-2 text-sm text-red-600 underline hover:text-red-500"
					>
						Go back
					</button>
				</div>
			</div>
		{:else if task}
			<div class="flex h-full gap-4">
				<!-- Task info panel -->
				<div class="w-80 flex-shrink-0 overflow-y-auto rounded-lg bg-white p-4 shadow">
					<h2 class="text-lg font-medium text-gray-900">Task Details</h2>

					<dl class="mt-4 space-y-3">
						<div>
							<dt class="text-xs font-medium text-gray-500">ClickUp ID</dt>
							<dd class="mt-1 text-sm text-gray-900">{task.clickup_task_id}</dd>
						</div>

						{#if task.description}
							<div>
								<dt class="text-xs font-medium text-gray-500">Description</dt>
								<dd class="mt-1 text-sm text-gray-900 whitespace-pre-wrap">{task.description}</dd>
							</div>
						{/if}

						{#if task.worktree_path}
							<div>
								<dt class="text-xs font-medium text-gray-500">Worktree Path</dt>
								<dd class="mt-1 text-xs font-mono text-gray-900 break-all">{task.worktree_path}</dd>
							</div>
						{/if}

						{#if task.started_at}
							<div>
								<dt class="text-xs font-medium text-gray-500">Started</dt>
								<dd class="mt-1 text-sm text-gray-900">
									{new Date(task.started_at).toLocaleString()}
								</dd>
							</div>
						{/if}

						{#if task.completed_at}
							<div>
								<dt class="text-xs font-medium text-gray-500">Completed</dt>
								<dd class="mt-1 text-sm text-gray-900">
									{new Date(task.completed_at).toLocaleString()}
								</dd>
							</div>
						{/if}
					</dl>
				</div>

				<!-- Terminal -->
				<div class="flex-1 overflow-hidden rounded-lg shadow">
					<Terminal taskId={task.id} onKill={handleKill} />
				</div>
			</div>
		{/if}
		</main>
	</div>
</div>

<!-- Delete Confirmation Modal -->
<ConfirmModal
	open={deleteModalOpen}
	title="Delete Task"
	message={task ? `Are you sure you want to delete "${task.name}"? This will also remove the git worktree if it exists. This action cannot be undone.` : ''}
	confirmText="Delete"
	cancelText="Cancel"
	danger={true}
	onConfirm={confirmDelete}
	onCancel={cancelDelete}
/>
