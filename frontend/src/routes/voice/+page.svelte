<script lang="ts">
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import VoiceRecorder from '$lib/components/voice/VoiceRecorder.svelte';
	import { projectStore } from '$lib/stores/project.svelte';
	import { getProject } from '$lib/api/projects';

	let sidebarCollapsed = $state(false);
	let projectName = $state('');

	onMount(async () => {
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}

		// Load current project name for display
		if (projectStore.currentProjectId) {
			try {
				const project = await getProject(projectStore.currentProjectId);
				if (project) {
					projectName = project.name;
				}
			} catch (e) {
				console.error('Failed to load project:', e);
			}
		}
	});

	$effect(() => {
		localStorage.setItem('sidebarCollapsed', String(sidebarCollapsed));
	});
</script>

<svelte:head>
	<title>Voice Assistant - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex h-screen bg-gray-50">
	<Sidebar bind:collapsed={sidebarCollapsed} />

	<main class="flex-1 overflow-auto">
		<div class="mx-auto max-w-5xl px-4 py-8 sm:px-6 lg:px-8">
			<!-- Header -->
			<div class="mb-8">
				<div class="flex items-center gap-3">
					<h1 class="text-2xl font-bold text-gray-900">Voice Assistant</h1>
					<span class="text-xs font-medium bg-amber-100 text-amber-700 px-2 py-1 rounded-full">
						EXPERIMENTAL
					</span>
					{#if projectName}
						<span class="text-xs font-medium bg-indigo-100 text-indigo-700 px-2 py-1 rounded-full">{projectName}</span>
					{/if}
				</div>
				<p class="mt-2 text-sm text-gray-500">
					Describe what you want to build using your voice and optionally share your screen for context.
					The Business Analyst agent will convert your description into actionable tasks.
				</p>
			</div>

			<!-- Voice Recorder Component -->
			<VoiceRecorder />
		</div>
	</main>
</div>
