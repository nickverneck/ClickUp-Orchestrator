// Workflow API

import { get, post, put } from './client';

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

export interface WorkflowResponse {
	status: WorkflowStatus;
	config: WorkflowConfig;
}

export async function getWorkflow(): Promise<WorkflowResponse> {
	return get<WorkflowResponse>('/workflow');
}

export async function updateWorkflow(config: WorkflowConfig): Promise<WorkflowResponse> {
	return put<WorkflowResponse>('/workflow', { config });
}

export async function setWorkflowStatus(status: WorkflowStatus): Promise<WorkflowResponse> {
	return put<WorkflowResponse>('/workflow/status', { status });
}

export async function startWorkflow(): Promise<WorkflowResponse> {
	return post<WorkflowResponse>('/workflow/start');
}

export async function pauseWorkflow(): Promise<WorkflowResponse> {
	return post<WorkflowResponse>('/workflow/pause');
}
