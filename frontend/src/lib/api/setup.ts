// Setup API for first-time configuration

import { get, post } from './client';

export interface SetupStatus {
	is_complete: boolean;
	has_api_key: boolean;
	has_list_selected: boolean;
	has_repo_configured: boolean;
	api_key_valid: boolean;
}

export interface SaveApiKeyResponse {
	success: boolean;
	valid: boolean;
	error?: string;
}

export async function getSetupStatus(): Promise<SetupStatus> {
	return get<SetupStatus>('/setup/status');
}

export async function saveApiKey(apiKey: string): Promise<SaveApiKeyResponse> {
	return post<SaveApiKeyResponse>('/setup/api-key', { api_key: apiKey });
}

export async function completeSetup(): Promise<{ success: boolean; error?: string }> {
	return post<{ success: boolean; error?: string }>('/setup/complete');
}
