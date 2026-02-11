<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import { useTasks } from '$lib/stores/tasks.svelte';
	import { projectStore } from '$lib/stores/project.svelte';
	import KanbanColumn from '$lib/components/kanban/KanbanColumn.svelte';
	import ConfirmModal from '$lib/components/ui/ConfirmModal.svelte';

	let projectId: number;
	let loading = $state(true);
	let error = $state<string | null>(null);
	let tasks: ReturnType<typeof useTasks>;

	// Delete confirmation modal state
	let deleteModalOpen = $state(false);
	let taskToDelete = $state<{ id: number; name: string } | null>(null);

	// Auto-refresh
	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(async () => {
		if (!$page.params.id) {
			error = 'Invalid project ID';
			loading = false;
			return;
		}
		projectId = parseInt($page.params.id, 10);
		if (isNaN(projectId)) {
			error = 'Invalid project ID';
			loading = false;
			return;
		}

		// Set current project
		projectStore.setProjectId(projectId);

		// Initialize tasks store with project scope
		tasks = useTasks(projectId);
		await tasks.loadTasks();
		await tasks.loadStats();
		loading = false;

		refreshInterval = setInterval(() => {
			tasks.loadTasks();
			tasks.loadStats();
		}, 5000);
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});

	function handleView(id: number) {
		goto(`/task/${id}`);
	}

	function handleStop(id: number) {
		tasks.stop(id);
	}

	function handleRestart(id: number) {
		tasks.restart(id);
	}

	function handleComplete(id: number) {
		tasks.complete(id);
	}

	function handleDelete(id: number) {
		const task = tasks.tasks.find((t) => t.id === id);
		if (task) {
			taskToDelete = { id: task.id, name: task.name };
			deleteModalOpen = true;
		}
	}

	function confirmDelete() {
		if (taskToDelete) {
			tasks.remove(taskToDelete.id);
		}
		deleteModalOpen = false;
		taskToDelete = null;
	}

	function cancelDelete() {
		deleteModalOpen = false;
		taskToDelete = null;
	}
</script>

<svelte:head>
	<title>Tasks - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex-1 overflow-auto">
	<div class="mx-auto max-w-full px-6 py-8">
		<div class="flex items-center justify-between mb-6">
			<div>
				<h1 class="text-2xl font-semibold text-gray-900">Tasks</h1>
				<p class="text-sm text-gray-500">Agent tasks for this project</p>
			</div>

			{#if tasks?.stats}
				<div class="flex items-center gap-4 text-sm">
					<span class="text-gray-600">
						Running: <span class="font-semibold text-green-600">{tasks.stats.running_processes}</span>
					</span>
					<span class="text-gray-600">
						In Progress: <span class="font-semibold text-blue-600">{tasks.stats.in_progress}</span>
					</span>
					<span class="text-gray-600">
						Completed: <span class="font-semibold text-green-600">{tasks.stats.completed}</span>
					</span>
					{#if tasks.stats.failed > 0}
						<span class="text-gray-600">
							Failed: <span class="font-semibold text-red-600">{tasks.stats.failed}</span>
						</span>
					{/if}
				</div>
			{/if}
		</div>

		{#if loading}
			<div class="flex items-center justify-center py-12">
				<svg class="h-8 w-8 animate-spin text-indigo-600" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
			</div>
		{:else if error}
			<div class="rounded-md bg-red-50 p-4">
				<p class="text-sm text-red-700">{error}</p>
			</div>
		{:else if tasks}
			{#if tasks.error}
				<div class="rounded-md bg-red-50 p-4 mb-4">
					<p class="text-sm text-red-700">{tasks.error}</p>
				</div>
			{/if}

			<!-- Kanban columns -->
			<div class="flex gap-4 overflow-x-auto pb-4">
				<KanbanColumn
					title="Queue"
					tasks={tasks.queued}
					color="gray"
					onView={handleView}
				/>

				<KanbanColumn
					title="In Development"
					tasks={tasks.inProgress}
					color="blue"
					onStop={handleStop}
					onView={handleView}
					onComplete={handleComplete}
				/>

				<KanbanColumn
					title="Stopped"
					tasks={tasks.stopped}
					color="yellow"
					onRestart={handleRestart}
					onView={handleView}
					onDelete={handleDelete}
				/>

				<KanbanColumn
					title="Completed"
					tasks={tasks.completed}
					color="green"
					onView={handleView}
					onDelete={handleDelete}
				/>

				{#if tasks.failed.length > 0}
					<KanbanColumn
						title="Failed"
						tasks={tasks.failed}
						color="red"
						onRestart={handleRestart}
						onView={handleView}
						onDelete={handleDelete}
					/>
				{/if}
			</div>

			{#if tasks.loading}
				<div class="absolute top-2 right-2">
					<svg class="h-5 w-5 animate-spin text-indigo-600" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
				</div>
			{/if}
		{/if}
	</div>
</div>

<!-- Delete Confirmation Modal -->
<ConfirmModal
	open={deleteModalOpen}
	title="Delete Task"
	message={taskToDelete ? `Are you sure you want to delete "${taskToDelete.name}"? This will also remove the git worktree if it exists. This action cannot be undone.` : ''}
	confirmText="Delete"
	cancelText="Cancel"
	danger={true}
	onConfirm={confirmDelete}
	onCancel={cancelDelete}
/>
