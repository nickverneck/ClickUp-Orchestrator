<script lang="ts">
	import { page } from '$app/stores';
	import { getProject } from '$lib/api/projects';
	import { projectStore } from '$lib/stores/project.svelte';
	import { useTasks } from '$lib/stores/tasks.svelte';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';

	let project = $state.raw(projectStore.project);
	let loading = $state(true);
	let error = $state<string | null>(null);
	const tasks = useTasks();
	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(async () => {
		if (!$page.params.id) {
			error = 'Invalid project ID';
			loading = false;
			return;
		}
		const projectId = parseInt($page.params.id, 10);
		if (isNaN(projectId)) {
			error = 'Invalid project ID';
			loading = false;
			return;
		}

		try {
			await projectStore.loadProject(projectId);
			project = projectStore.project;

			tasks.setProjectId(projectId);
			await tasks.loadStats();
			refreshInterval = setInterval(() => tasks.loadStats(), 10000);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load project';
		} finally {
			loading = false;
		}
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});

	function goToTasks() {
		goto(`/projects/${project?.id}/tasks`);
	}

	function goToWorkflows() {
		goto(`/projects/${project?.id}/workflows`);
	}

	function goToSettings() {
		goto(`/projects/${project?.id}/settings`);
	}
</script>

<svelte:head>
	<title>{project?.name || 'Project'} - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex-1 overflow-auto">
	<!-- Header -->
	<div class="bg-white shadow">
		<div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-3xl font-bold tracking-tight text-gray-900">{project?.name || 'Loading...'}</h1>
					{#if project?.description}
						<p class="mt-2 text-sm text-gray-600">{project.description}</p>
					{/if}
				</div>
				<div class="flex gap-2">
					<button
						onclick={goToWorkflows}
						class="inline-flex items-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500"
					>
						<svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
						</svg>
						New Workflow
					</button>
					<button
						onclick={goToSettings}
						class="inline-flex items-center rounded-md border border-gray-300 px-4 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50"
					>
						<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
						</svg>
						Settings
					</button>
				</div>
			</div>
		</div>
	</div>

	<!-- Content -->
	<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
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
		{:else if project}
			<!-- Task Stats Summary -->
			{#if tasks?.stats}
				<button onclick={goToTasks} class="w-full mb-6 rounded-lg bg-white p-6 shadow-sm hover:shadow-md transition-shadow text-left">
					<div class="flex items-center justify-between">
						<h3 class="text-sm font-semibold text-gray-900">Agent Tasks</h3>
						<span class="text-xs text-indigo-600 font-medium">View Kanban Board &rarr;</span>
					</div>
					<div class="mt-4 grid grid-cols-5 gap-4">
						<div class="text-center">
							<div class="text-2xl font-bold text-gray-600">{tasks.stats.queued}</div>
							<div class="text-xs text-gray-500">Queued</div>
						</div>
						<div class="text-center">
							<div class="text-2xl font-bold text-blue-600">{tasks.stats.in_progress}</div>
							<div class="text-xs text-gray-500">In Progress</div>
						</div>
						<div class="text-center">
							<div class="text-2xl font-bold text-green-600">{tasks.stats.running_processes}</div>
							<div class="text-xs text-gray-500">Running</div>
						</div>
						<div class="text-center">
							<div class="text-2xl font-bold text-green-700">{tasks.stats.completed}</div>
							<div class="text-xs text-gray-500">Completed</div>
						</div>
						<div class="text-center">
							<div class="text-2xl font-bold {tasks.stats.failed > 0 ? 'text-red-600' : 'text-gray-400'}">{tasks.stats.failed}</div>
							<div class="text-xs text-gray-500">Failed</div>
						</div>
					</div>
				</button>
			{/if}

			<!-- Project Overview -->
			<div class="grid grid-cols-1 gap-6 sm:grid-cols-3">
				<!-- Repository Info -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<div class="flex items-center">
						<svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
						</svg>
						<h3 class="ml-2 text-sm font-semibold text-gray-900">Repository</h3>
					</div>
					<div class="mt-4 space-y-2 text-sm text-gray-600">
						<p><span class="font-medium">Path:</span> {project.repo_path}</p>
						<p><span class="font-medium">Branch:</span> {project.dev_branch}</p>
						{#if project.github_url}
							<p><span class="font-medium">GitHub:</span> <a href={project.github_url} target="_blank" class="text-indigo-600 hover:text-indigo-500">{project.github_url}</a></p>
						{/if}
					</div>
				</div>

				<!-- ClickUp Configuration -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<div class="flex items-center">
						<svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2m-6 9l2 2 4-4" />
						</svg>
						<h3 class="ml-2 text-sm font-semibold text-gray-900">ClickUp</h3>
					</div>
					<div class="mt-4 space-y-2 text-sm text-gray-600">
						{#if project.clickup_list_id}
							<p><span class="font-medium">List ID:</span> {project.clickup_list_id}</p>
						{:else}
							<p class="text-gray-500">Not configured</p>
						{/if}
					</div>
				</div>

				<!-- Agent Configuration -->
				<div class="rounded-lg bg-white p-6 shadow-sm">
					<div class="flex items-center">
						<svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
						</svg>
						<h3 class="ml-2 text-sm font-semibold text-gray-900">Agent</h3>
					</div>
					<div class="mt-4 space-y-2 text-sm text-gray-600">
						<p><span class="font-medium">Model:</span> {project.agent_model}</p>
						<p><span class="font-medium">Parallel Limit:</span> {project.parallel_limit}</p>
					</div>
				</div>
			</div>
		{/if}
	</main>
</div>
