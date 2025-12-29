// Git repository API

import { get, post } from './client';

export interface ValidatePathResponse {
	valid: boolean;
	error?: string;
}

export interface BranchesResponse {
	branches: string[];
	current?: string;
}

export interface DetectPathResponse {
	found: boolean;
	path?: string;
}

export async function validatePath(path: string): Promise<ValidatePathResponse> {
	return post<ValidatePathResponse>('/git/validate-path', { path });
}

export async function getBranches(path: string): Promise<BranchesResponse> {
	return get<BranchesResponse>(`/git/branches?path=${encodeURIComponent(path)}`);
}

export async function fetchRepo(path: string): Promise<{ success: boolean }> {
	return post<{ success: boolean }>('/git/fetch', { path });
}

export async function detectPath(markerFilename: string): Promise<DetectPathResponse> {
	return post<DetectPathResponse>('/git/detect-path', { marker_filename: markerFilename });
}
