// Tasks store using Svelte 5 runes

import { getTasks, getTaskStats, stopTask, restartTask, completeTask, deleteTask, type Task, type TaskStats } from '$lib/api/tasks';

// Tasks state
let tasks = $state<Task[]>([]);
let stats = $state<TaskStats | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

export function useTasks() {
	async function loadTasks() {
		loading = true;
		error = null;
		try {
			tasks = await getTasks();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load tasks';
		} finally {
			loading = false;
		}
	}

	async function loadStats() {
		try {
			stats = await getTaskStats();
		} catch (e) {
			console.error('Failed to load stats:', e);
		}
	}

	async function stop(id: number) {
		try {
			const updated = await stopTask(id);
			tasks = tasks.map((t) => (t.id === id ? updated : t));
			await loadStats();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to stop task';
		}
	}

	async function restart(id: number) {
		try {
			const updated = await restartTask(id);
			tasks = tasks.map((t) => (t.id === id ? updated : t));
			await loadStats();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to restart task';
		}
	}

	async function complete(id: number) {
		try {
			const updated = await completeTask(id);
			tasks = tasks.map((t) => (t.id === id ? updated : t));
			await loadStats();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to mark task as complete';
		}
	}

	async function remove(id: number) {
		try {
			await deleteTask(id);
			tasks = tasks.filter((t) => t.id !== id);
			await loadStats();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete task';
		}
	}

	// Group tasks by status
	function getTasksByStatus(status: string): Task[] {
		return tasks.filter((t) => t.status === status);
	}

	return {
		get tasks() {
			return tasks;
		},
		get stats() {
			return stats;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		loadTasks,
		loadStats,
		stop,
		restart,
		complete,
		remove,
		getTasksByStatus,
		get queued() {
			return getTasksByStatus('queued');
		},
		get inProgress() {
			return getTasksByStatus('in_progress');
		},
		get stopped() {
			return getTasksByStatus('stopped');
		},
		get completed() {
			return getTasksByStatus('completed');
		},
		get failed() {
			return getTasksByStatus('failed');
		}
	};
}
