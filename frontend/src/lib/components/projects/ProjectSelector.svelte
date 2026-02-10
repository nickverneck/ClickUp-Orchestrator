<script lang="ts">
	import { useCurrentProject } from '$lib/stores/project.svelte';
	import type { ProjectListItem } from '$lib/api/projects';

	interface Props {
		projects?: ProjectListItem[];
		onSelect?: (id: number) => void;
	}

	let { projects = [], onSelect }: Props = $props();
	const projectStore = useCurrentProject();

	let isOpen = $state(false);

	const currentProject = $derived(
		projects.find((p) => p.id === projectStore.currentProjectId)
	);
</script>

<div class="relative">
	<button
		onclick={() => (isOpen = !isOpen)}
		class="w-full flex items-center justify-between px-3 py-2 rounded-lg border border-gray-300 bg-white text-left text-sm font-medium text-gray-900 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500"
	>
		<span class="truncate">
			{currentProject?.name || 'Select a project'}
		</span>
		<svg
			class="h-4 w-4 flex-shrink-0 text-gray-400 transition-transform {isOpen
				? 'rotate-180'
				: ''}"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
		</svg>
	</button>

	{#if isOpen}
		<div
			class="absolute z-50 mt-1 w-full rounded-lg border border-gray-300 bg-white shadow-lg"
			role="listbox"
		>
			<div class="max-h-64 overflow-y-auto py-1">
				{#if projects.length === 0}
					<div class="px-3 py-2 text-sm text-gray-500 text-center">No projects available</div>
				{:else}
					{#each projects as project (project.id)}
						<button
							type="button"
							onclick={() => {
								projectStore.setProjectId(project.id);
								onSelect?.(project.id);
								isOpen = false;
							}}
							class="w-full text-left px-3 py-2 text-sm text-gray-900 hover:bg-indigo-50 transition-colors {projectStore.currentProjectId ===
							project.id
								? 'bg-indigo-100 font-medium'
								: ''}"
							role="option"
							aria-selected={projectStore.currentProjectId === project.id}
						>
							<div class="flex items-center justify-between">
								<span>{project.name}</span>
								{#if projectStore.currentProjectId === project.id}
									<svg class="h-4 w-4 text-indigo-600" fill="currentColor" viewBox="0 0 20 20">
										<path
											fill-rule="evenodd"
											d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
											clip-rule="evenodd"
										/>
									</svg>
								{/if}
							</div>
							<div class="text-xs text-gray-500">{project.description || project.repo_path}</div>
						</button>
					{/each}
				{/if}
			</div>
		</div>
	{/if}
</div>
