// Setup API for first-time configuration

import { get, post } from './client';

export interface SetupStatus {
	is_complete: boolean;
	has_project: boolean;
}

export async function getSetupStatus(): Promise<SetupStatus> {
	return get<SetupStatus>('/setup/status');
}
