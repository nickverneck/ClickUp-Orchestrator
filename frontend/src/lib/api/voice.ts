// Voice Assistant API

import { post, del } from './client';

export type AgentType = 'claude' | 'codex' | 'gemini';

export interface AgentOption {
	value: AgentType;
	label: string;
	description: string;
}

export const AGENT_OPTIONS: AgentOption[] = [
	{
		value: 'claude',
		label: 'Claude',
		description: 'Anthropic Claude Code CLI'
	},
	{
		value: 'codex',
		label: 'Codex',
		description: 'OpenAI Codex CLI'
	},
	{
		value: 'gemini',
		label: 'Gemini',
		description: 'Google Gemini CLI'
	}
];

export interface SaveScreenshotResponse {
	filepath: string;
	filename: string;
}

export interface GenerateTasksResponse {
	success: boolean;
	message: string;
	session_id?: string;
}

export interface ClearScreenshotsResponse {
	success: boolean;
	message: string;
}

/**
 * Save a screenshot to the temp_imgs folder in the target repo
 * @param imageData Base64 encoded image data (can include data URL prefix)
 * @param filename Optional filename
 */
export async function saveScreenshot(
	imageData: string,
	filename?: string
): Promise<SaveScreenshotResponse> {
	return post<SaveScreenshotResponse>('/voice/screenshot', {
		image_data: imageData,
		filename
	});
}

/**
 * Generate tasks by spawning the BA agent with transcript and screenshots
 * @param transcript The voice transcription text
 * @param screenshots Array of screenshot filepaths (relative to repo)
 * @param agent Which agent to use (claude, codex, gemini)
 */
export async function generateTasks(
	transcript: string,
	screenshots: string[],
	agent: AgentType = 'claude'
): Promise<GenerateTasksResponse> {
	return post<GenerateTasksResponse>('/voice/generate-tasks', {
		transcript,
		screenshots,
		agent
	});
}

/**
 * Clear all screenshots from the temp_imgs folder
 */
export async function clearScreenshots(): Promise<ClearScreenshotsResponse> {
	return del<ClearScreenshotsResponse>('/voice/screenshots');
}
