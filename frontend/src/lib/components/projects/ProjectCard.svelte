<script lang="ts">
	import type { ProjectListItem } from '$lib/api/projects';

	interface Props {
		project: ProjectListItem;
		onclick?: () => void;
	}

	let { project, onclick }: Props = $props();
</script>

<button
	class="block w-full text-left p-6 rounded-lg border border-gray-200 bg-white hover:shadow-lg hover:border-indigo-300 transition-all"
	{onclick}
>
	<div class="flex items-start justify-between mb-2">
		<h3 class="text-lg font-semibold text-gray-900">{project.name}</h3>
		<span
			class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium
			{project.status === 'active'
				? 'bg-green-100 text-green-800'
				: 'bg-gray-100 text-gray-800'}"
		>
			{project.status}
		</span>
	</div>

	{#if project.description}
		<p class="text-sm text-gray-600 mb-4 line-clamp-2">{project.description}</p>
	{/if}

	<div class="space-y-2 text-sm text-gray-500">
		<div class="flex items-center gap-2">
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
				/>
			</svg>
			<span class="font-mono text-xs">{project.dev_branch}</span>
		</div>
		<div class="flex items-center gap-2">
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
				/>
			</svg>
			<span class="truncate text-xs">{project.repo_path}</span>
		</div>
	</div>

	<div class="flex gap-4 mt-4 pt-4 border-t border-gray-100 text-xs">
		<div>
			<div class="text-gray-500">Workflows</div>
			<div class="font-semibold text-gray-900">{project.workflow_count}</div>
		</div>
		<div>
			<div class="text-gray-500">Active Tasks</div>
			<div class="font-semibold text-gray-900">{project.active_task_count}</div>
		</div>
	</div>
</button>
