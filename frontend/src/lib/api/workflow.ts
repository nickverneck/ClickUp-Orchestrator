// Workflow API

import { del, get, post, put } from './client';

export type WorkflowStatus = 'running' | 'paused';

export interface WorkflowPosition {
	x: number;
	y: number;
}

export interface WorkflowNode {
	id: string;
	type: string;
	label: string;
	position: WorkflowPosition;
	settings: Record<string, unknown>;
}

export interface WorkflowAction {
	id: string;
	type: string;
	settings: Record<string, unknown>;
}

export interface WorkflowEdge {
	id: string;
	source: string;
	target: string;
	output?: string | null;
	actions: WorkflowAction[];
}

export interface WorkflowConfig {
	version: number;
	nodes: WorkflowNode[];
	edges: WorkflowEdge[];
}

export interface WorkflowSummary {
	id: number;
	name: string;
	status: WorkflowStatus;
}

export interface WorkflowResponse {
	id: number;
	name: string;
	status: WorkflowStatus;
	config: WorkflowConfig;
}

export async function getWorkflows(): Promise<WorkflowSummary[]> {
	return get<WorkflowSummary[]>('/workflows');
}

export async function createWorkflow(name?: string): Promise<WorkflowResponse> {
	return post<WorkflowResponse>('/workflows', name ? { name } : undefined);
}

export async function getWorkflow(id: number): Promise<WorkflowResponse> {
	return get<WorkflowResponse>(`/workflows/${id}`);
}

export async function updateWorkflow(
	id: number,
	config: WorkflowConfig,
	name?: string
): Promise<WorkflowResponse> {
	return put<WorkflowResponse>(`/workflows/${id}`, { config, name });
}

export async function deleteWorkflow(id: number): Promise<{ success: boolean }> {
	return del<{ success: boolean }>(`/workflows/${id}`);
}

export async function setWorkflowStatus(
	id: number,
	status: WorkflowStatus
): Promise<WorkflowResponse> {
	return put<WorkflowResponse>(`/workflows/${id}/status`, { status });
}

export async function startWorkflow(id: number): Promise<WorkflowResponse> {
	return post<WorkflowResponse>(`/workflows/${id}/start`);
}

export async function pauseWorkflow(id: number): Promise<WorkflowResponse> {
	return post<WorkflowResponse>(`/workflows/${id}/pause`);
}
