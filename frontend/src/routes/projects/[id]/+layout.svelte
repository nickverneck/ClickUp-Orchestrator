<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import { projectStore } from '$lib/stores/project.svelte';

	let { children } = $props();
	let sidebarCollapsed = $state(false);

	onMount(() => {
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}

		// Ensure project is loaded when navigating to project routes
		const id = parseInt($page.params.id, 10);
		if (!isNaN(id) && projectStore.currentProjectId !== id) {
			projectStore.setProjectId(id);
		}
	});

	$effect(() => {
		localStorage.setItem('sidebarCollapsed', String(sidebarCollapsed));
	});
</script>

<div class="flex h-screen bg-gray-50">
	<Sidebar bind:collapsed={sidebarCollapsed} />
	<div class="flex-1 flex flex-col overflow-hidden">
		{@render children()}
	</div>
</div>
