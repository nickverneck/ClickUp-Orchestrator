// Tasks API

import { get, post, del } from './client';

export interface Task {
	id: number;
	clickup_task_id: string;
	name: string;
	description?: string;
	priority?: number;
	status: string;
	worktree_path?: string;
	time_spent_ms: number;
	started_at?: string;
	completed_at?: string;
	is_running: boolean;
}

export interface TaskStats {
	queued: number;
	in_progress: number;
	stopped: number;
	completed: number;
	failed: number;
	running_processes: number;
}

export async function getTasks(status?: string): Promise<Task[]> {
	const query = status ? `?status=${encodeURIComponent(status)}` : '';
	return get<Task[]>(`/tasks${query}`);
}

export async function getTask(id: number): Promise<Task> {
	return get<Task>(`/tasks/${id}`);
}

export async function getTaskStats(): Promise<TaskStats> {
	return get<TaskStats>('/tasks/stats');
}

export async function stopTask(id: number): Promise<Task> {
	return post<Task>(`/tasks/${id}/stop`);
}

export async function restartTask(id: number): Promise<Task> {
	return post<Task>(`/tasks/${id}/restart`);
}

export async function completeTask(id: number): Promise<Task> {
	return post<Task>(`/tasks/${id}/complete`);
}

export async function deleteTask(id: number): Promise<{ success: boolean; message: string }> {
	return del<{ success: boolean; message: string }>(`/tasks/${id}`);
}
