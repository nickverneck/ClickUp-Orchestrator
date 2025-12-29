// Settings API

import { get, put } from './client';

export interface SettingsResponse {
	settings: Record<string, string>;
}

export async function getSettings(): Promise<Record<string, string>> {
	const response = await get<SettingsResponse>('/settings');
	return response.settings;
}

export async function updateSettings(settings: Record<string, string>): Promise<Record<string, string>> {
	const response = await put<SettingsResponse>('/settings', { settings });
	return response.settings;
}
