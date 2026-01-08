<script lang="ts">
	import type { Task } from '$lib/api/tasks';
	import TaskCard from './TaskCard.svelte';

	interface Props {
		title: string;
		tasks: Task[];
		color: 'gray' | 'blue' | 'yellow' | 'green' | 'red';
		onStop?: (id: number) => void;
		onRestart?: (id: number) => void;
		onView?: (id: number) => void;
		onDelete?: (id: number) => void;
		onComplete?: (id: number) => void;
	}

	let { title, tasks, color, onStop, onRestart, onView, onDelete, onComplete }: Props = $props();

	const colorClasses = {
		gray: 'bg-gray-100 border-gray-300',
		blue: 'bg-blue-50 border-blue-300',
		yellow: 'bg-yellow-50 border-yellow-300',
		green: 'bg-green-50 border-green-300',
		red: 'bg-red-50 border-red-300'
	};

	const headerColors = {
		gray: 'text-gray-700',
		blue: 'text-blue-700',
		yellow: 'text-yellow-700',
		green: 'text-green-700',
		red: 'text-red-700'
	};
</script>

<div class="flex-1 min-w-[280px] max-w-[400px]">
	<div class="rounded-t-lg border-t-4 {colorClasses[color]} p-3">
		<div class="flex items-center justify-between">
			<h3 class="text-sm font-semibold {headerColors[color]}">{title}</h3>
			<span class="inline-flex items-center justify-center rounded-full bg-white px-2 py-0.5 text-xs font-medium text-gray-600 shadow-sm">
				{tasks.length}
			</span>
		</div>
	</div>

	<div class="space-y-3 rounded-b-lg border border-t-0 border-gray-200 bg-gray-50 p-3 min-h-[400px]">
		{#each tasks as task (task.id)}
			<TaskCard {task} {onStop} {onRestart} {onView} {onDelete} {onComplete} />
		{:else}
			<div class="flex items-center justify-center h-32 text-sm text-gray-400">
				No tasks
			</div>
		{/each}
	</div>
</div>
