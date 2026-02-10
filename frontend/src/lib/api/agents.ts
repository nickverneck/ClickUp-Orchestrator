// Agents API client

import { get } from './client';

export interface AgentModel {
	id: string;
	provider: string;
	name: string;
}

export async function getOpenCodeModels(): Promise<AgentModel[]> {
	return get<AgentModel[]>('/agents/opencode/models');
}
