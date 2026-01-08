<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import Terminal from '$lib/components/terminal/Terminal.svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import ConfirmModal from '$lib/components/ui/ConfirmModal.svelte';
	import {
		getTask,
		getTaskLogs,
		stopTask,
		restartTask,
		deleteTask,
		type Task,
		type TaskLogEntry
	} from '$lib/api/tasks';

	let task = $state<Task | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let sidebarCollapsed = $state(false);
	let deleteModalOpen = $state(false);
	let activeTab = $state<'terminal' | 'logs'>('terminal');
	let logs = $state<TaskLogEntry[]>([]);
	let logsLoading = $state(false);
	let logsError = $state<string | null>(null);
	let legacyOutputLog = $state<string | null>(null);
	let logsInterval: ReturnType<typeof setInterval> | null = null;

	const taskId = $derived(parseInt($page.params.id ?? '0', 10));

	onMount(() => {
		loadTask();
		// Load sidebar state from localStorage
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}
	});

	onDestroy(() => {
		stopLogsPolling();
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

	async function loadLogs() {
		logsLoading = true;
		logsError = null;
		try {
			const response = await getTaskLogs(taskId);
			logs = response.logs;
			legacyOutputLog = response.legacy_output_log ?? null;
		} catch (e) {
			logsError = e instanceof Error ? e.message : 'Failed to load logs';
		} finally {
			logsLoading = false;
		}
	}

	function startLogsPolling() {
		if (logsInterval) return;
		logsInterval = setInterval(loadLogs, 5000);
	}

	function stopLogsPolling() {
		if (logsInterval) {
			clearInterval(logsInterval);
			logsInterval = null;
		}
	}

	$effect(() => {
		if (activeTab === 'logs') {
			loadLogs();
			startLogsPolling();
		} else {
			stopLogsPolling();
		}
	});

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

	function formatLogTime(timestamp: string): string {
		return new Date(timestamp).toLocaleString();
	}

	function getLogBadge(log: TaskLogEntry): { text: string; class: string } {
		switch (log.event_type) {
			case 'status':
				return { text: 'Status', class: 'bg-blue-100 text-blue-800' };
			case 'clickup':
				return { text: 'ClickUp', class: 'bg-green-100 text-green-800' };
			case 'system':
				return { text: 'System', class: 'bg-yellow-100 text-yellow-800' };
			case 'output':
				return log.is_stderr
					? { text: 'Stderr', class: 'bg-red-100 text-red-800' }
					: { text: 'Output', class: 'bg-gray-100 text-gray-700' };
			default:
				return { text: 'Log', class: 'bg-gray-100 text-gray-600' };
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
				<div class="flex-1 overflow-hidden rounded-lg shadow bg-white flex flex-col">
					<div class="flex items-center justify-between border-b bg-gray-50">
						<div class="flex">
							<button
								class="px-4 py-2 text-sm font-medium border-b-2 {activeTab === 'terminal' ? 'border-gray-900 text-gray-900' : 'border-transparent text-gray-500 hover:text-gray-700'}"
								onclick={() => (activeTab = 'terminal')}
							>
								Terminal
							</button>
							<button
								class="px-4 py-2 text-sm font-medium border-b-2 {activeTab === 'logs' ? 'border-gray-900 text-gray-900' : 'border-transparent text-gray-500 hover:text-gray-700'}"
								onclick={() => (activeTab = 'logs')}
							>
								Logs
							</button>
						</div>
						{#if activeTab === 'logs'}
							<button
								class="mr-3 rounded-md border border-gray-300 bg-white px-2.5 py-1 text-xs font-medium text-gray-700 hover:bg-gray-100 disabled:opacity-50"
								disabled={logsLoading}
								onclick={loadLogs}
							>
								Refresh
							</button>
						{/if}
					</div>

					{#if activeTab === 'terminal'}
						<div class="flex-1 overflow-hidden">
							<Terminal taskId={task.id} onKill={handleKill} />
						</div>
					{:else}
						<div class="flex-1 overflow-y-auto bg-gray-900 px-4 py-3 text-xs text-gray-200 font-mono">
							{#if logsLoading}
								<div class="text-gray-400">Loading logs...</div>
							{:else if logsError}
								<div class="text-red-400">{logsError}</div>
							{:else if logs.length === 0 && legacyOutputLog}
								<div class="text-gray-400">Legacy output (timestamps unavailable).</div>
								<pre class="mt-2 whitespace-pre-wrap text-gray-100">{legacyOutputLog}</pre>
							{:else if logs.length === 0}
								<div class="text-gray-400">No logs yet.</div>
							{:else}
								<div class="space-y-2">
									{#each logs as log}
										{#key log.id}
											<div class="flex gap-3 items-start">
												<span class="text-gray-400 whitespace-nowrap">
													{formatLogTime(log.created_at)}
												</span>
												{@const badge = getLogBadge(log)}
												<span class="inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-semibold uppercase {badge.class}">
													{badge.text}
												</span>
												<span class={log.is_stderr ? 'text-red-300 whitespace-pre-wrap' : 'text-gray-100 whitespace-pre-wrap'}>
													{log.message}
												</span>
											</div>
										{/key}
									{/each}
								</div>
							{/if}
						</div>
					{/if}
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
