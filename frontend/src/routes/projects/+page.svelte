<script lang="ts">
	import { onMount } from 'svelte';
	import { listProjects, type ProjectListItem } from '$lib/api/projects';
	import { projectStore } from '$lib/stores/project.svelte';
	import { goto } from '$app/navigation';

	let projects = $state<ProjectListItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		await loadProjects();
	});

	async function loadProjects() {
		loading = true;
		error = null;
		try {
			projects = await listProjects();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load projects';
		} finally {
			loading = false;
		}
	}

	function selectProject(projectId: number) {
		projectStore.setProjectId(projectId);
		goto(`/projects/${projectId}`);
	}

	function createNewProject() {
		goto('/projects/new');
	}
</script>

<svelte:head>
	<title>Projects - ClickUp Orchestrator</title>
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<!-- Header -->
	<div class="bg-white shadow">
		<div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between">
				<h1 class="text-3xl font-bold tracking-tight text-gray-900">Projects</h1>
				<button
					onclick={createNewProject}
					class="inline-flex items-center rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500"
				>
					<svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
					</svg>
					New Project
				</button>
			</div>
		</div>
	</div>

	<!-- Content -->
	<main class="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
		{#if loading}
			<!-- Loading state -->
			<div class="flex items-center justify-center py-12">
				<div class="text-center">
					<svg class="mx-auto h-12 w-12 animate-spin text-indigo-600" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					<p class="mt-4 text-gray-500">Loading projects...</p>
				</div>
			</div>
		{:else if error}
			<!-- Error state -->
			<div class="rounded-md bg-red-50 p-4">
				<div class="flex">
					<svg class="h-5 w-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
						<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
					</svg>
					<div class="ml-3">
						<h3 class="text-sm font-medium text-red-800">Error loading projects</h3>
						<p class="mt-2 text-sm text-red-700">{error}</p>
						<button
							onclick={loadProjects}
							class="mt-4 inline-flex items-center rounded-md bg-red-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-red-500"
						>
							Retry
						</button>
					</div>
				</div>
			</div>
		{:else if projects.length === 0}
			<!-- Empty state -->
			<div class="text-center py-12">
				<svg class="mx-auto h-16 w-16 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4" />
				</svg>
				<h2 class="mt-4 text-2xl font-bold tracking-tight text-gray-900">Let's start your first project</h2>
				<p class="mt-2 text-base text-gray-600">Create a project to manage your workflows and connect to ClickUp.</p>
				<button
					onclick={createNewProject}
					class="mt-6 inline-flex items-center rounded-lg bg-indigo-600 px-6 py-3 text-base font-semibold text-white shadow-sm hover:bg-indigo-500 transition-colors"
				>
					<svg class="mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
					</svg>
					Create your first project
				</button>
			</div>
		{:else}
			<!-- Projects grid -->
			<div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
				{#each projects as project (project.id)}
					<button
						onclick={() => selectProject(project.id)}
						class="group relative overflow-hidden rounded-lg border border-gray-200 bg-white p-6 shadow-sm transition-all hover:border-indigo-500 hover:shadow-md"
					>
						<div class="absolute inset-0 bg-gradient-to-r from-indigo-50 to-transparent opacity-0 transition-opacity group-hover:opacity-100"></div>
						<div class="relative">
							<!-- Status badge -->
							{#if project.status === 'archived'}
								<div class="mb-4 inline-block rounded-full bg-gray-100 px-3 py-1 text-xs font-semibold text-gray-700">
									Archived
								</div>
							{:else}
								<div class="mb-4 inline-block rounded-full bg-green-100 px-3 py-1 text-xs font-semibold text-green-700">
									Active
								</div>
							{/if}

							<!-- Project name -->
							<h3 class="mt-2 text-lg font-semibold text-gray-900">{project.name}</h3>

							<!-- Description -->
							{#if project.description}
								<p class="mt-1 text-sm text-gray-600 line-clamp-2">{project.description}</p>
							{/if}

							<!-- Project info -->
							<div class="mt-4 space-y-2 text-sm text-gray-600">
								<div class="flex items-center">
									<svg class="mr-2 h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12a9 9 0 019-9 9.75 9.75 0 016.74 2.74L21 8" />
									</svg>
									<span class="truncate">{project.repo_path}</span>
								</div>
								<div class="flex items-center">
									<svg class="mr-2 h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3v-6" />
									</svg>
									<span>{project.dev_branch}</span>
								</div>
							</div>

							<!-- Stats -->
							<div class="mt-4 flex items-center gap-4 border-t border-gray-200 pt-4 text-sm">
								<div class="text-center">
									<p class="text-2xl font-semibold text-gray-900">{project.workflow_count}</p>
									<p class="text-xs text-gray-500">Workflows</p>
								</div>
								<div class="text-center">
									<p class="text-2xl font-semibold text-gray-900">{project.active_task_count}</p>
									<p class="text-xs text-gray-500">Active Tasks</p>
								</div>
							</div>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</main>
</div>

<style>
	:global(body) {
		background-color: #f9fafb;
	}
</style>
