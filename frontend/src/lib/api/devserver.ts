// Dev Server API client

import { get, post } from './client';

export interface DetectResponse {
	found: boolean;
	package_json_dir: string | null;
	package_manager: string | null;
	has_dev_script: boolean;
}

export interface StartResponse {
	success: boolean;
	pid: number | null;
	error: string | null;
}

export interface StopResponse {
	success: boolean;
	error: string | null;
}

export interface StatusResponse {
	running: boolean;
}

export async function detectDevServer(repoPath: string): Promise<DetectResponse> {
	return post<DetectResponse>('/devserver/detect', { repo_path: repoPath });
}

export async function startDevServer(
	repoPath: string,
	packageJsonDir: string,
	packageManager: string
): Promise<StartResponse> {
	return post<StartResponse>('/devserver/start', {
		repo_path: repoPath,
		package_json_dir: packageJsonDir,
		package_manager: packageManager
	});
}

export async function stopDevServer(repoPath: string): Promise<StopResponse> {
	return post<StopResponse>('/devserver/stop', { repo_path: repoPath });
}

export async function getDevServerStatus(repoPath: string): Promise<StatusResponse> {
	return get<StatusResponse>(`/devserver/status?repo_path=${encodeURIComponent(repoPath)}`);
}
