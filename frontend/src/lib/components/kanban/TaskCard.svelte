<script lang="ts">
	import type { Task } from '$lib/api/tasks';

	interface Props {
		task: Task;
		onStop?: (id: number) => void;
		onRestart?: (id: number) => void;
		onView?: (id: number) => void;
		onDelete?: (id: number) => void;
		onComplete?: (id: number) => void;
	}

	let { task, onStop, onRestart, onView, onDelete, onComplete }: Props = $props();

	function formatDuration(ms: number): string {
		const seconds = Math.floor(ms / 1000);
		const minutes = Math.floor(seconds / 60);
		const hours = Math.floor(minutes / 60);

		if (hours > 0) {
			return `${hours}h ${minutes % 60}m`;
		} else if (minutes > 0) {
			return `${minutes}m ${seconds % 60}s`;
		} else {
			return `${seconds}s`;
		}
	}

	function getPriorityBadge(priority?: number): { text: string; class: string } {
		switch (priority) {
			case 1:
				return { text: 'Urgent', class: 'bg-red-100 text-red-800' };
			case 2:
				return { text: 'High', class: 'bg-orange-100 text-orange-800' };
			case 3:
				return { text: 'Normal', class: 'bg-blue-100 text-blue-800' };
			case 4:
				return { text: 'Low', class: 'bg-gray-100 text-gray-800' };
			default:
				return { text: 'None', class: 'bg-gray-100 text-gray-500' };
		}
	}

	const priority = $derived(getPriorityBadge(task.priority));
</script>

<div class="rounded-lg bg-white p-4 shadow hover:shadow-md transition-shadow">
	<div class="flex items-start justify-between">
		<div class="flex-1 min-w-0">
			<h4 class="text-sm font-medium text-gray-900 truncate">{task.name}</h4>
			<p class="mt-1 text-xs text-gray-500 truncate">
				{task.clickup_task_id}
			</p>
		</div>
		<span class="ml-2 inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {priority.class}">
			{priority.text}
		</span>
	</div>

	{#if task.description}
		<p class="mt-2 text-xs text-gray-600 line-clamp-2">
			{task.description}
		</p>
	{/if}

	<div class="mt-3 flex items-center justify-between">
		<div class="flex items-center gap-2">
			{#if task.is_running}
				<span class="flex items-center gap-1 text-xs text-green-600">
					<span class="relative flex h-2 w-2">
						<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
						<span class="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
					</span>
					Running
				</span>
			{/if}
			{#if task.time_spent_ms > 0}
				<span class="text-xs text-gray-500">
					{formatDuration(task.time_spent_ms)}
				</span>
			{/if}
		</div>

		<div class="flex items-center gap-1">
			{#if onView}
				<button
					onclick={() => onView(task.id)}
					class="p-1 text-gray-400 hover:text-gray-600"
					title="View Terminal"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
					</svg>
				</button>
			{/if}

			{#if task.status === 'in_progress' && onStop}
				<button
					onclick={() => onStop(task.id)}
					class="p-1 text-red-400 hover:text-red-600"
					title="Stop"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
					</svg>
				</button>
			{/if}

			{#if task.status === 'in_progress' && onComplete && !task.is_running}
				<button
					onclick={() => onComplete(task.id)}
					class="p-1 text-green-400 hover:text-green-600"
					title="Mark Complete"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</button>
			{/if}

			{#if (task.status === 'stopped' || task.status === 'failed') && onRestart}
				<button
					onclick={() => onRestart(task.id)}
					class="p-1 text-green-400 hover:text-green-600"
					title="Restart"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
					</svg>
				</button>
			{/if}

			{#if onDelete && (task.status === 'stopped' || task.status === 'failed' || task.status === 'completed')}
				<button
					onclick={() => onDelete(task.id)}
					class="p-1 text-gray-400 hover:text-red-600"
					title="Delete"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
					</svg>
				</button>
			{/if}
		</div>
	</div>
</div>
