// ClickUp hierarchy browser API

import { get } from './client';

export interface Team {
	id: string;
	name: string;
	color?: string;
	avatar?: string;
}

export interface Space {
	id: string;
	name: string;
	private: boolean;
	color?: string;
}

export interface Folder {
	id: string;
	name: string;
	hidden: boolean;
}

export interface List {
	id: string;
	name: string;
	content?: string;
}

export interface Status {
	id?: string;
	status: string;
	color?: string;
	type?: string;
	orderindex?: number;
}

function buildUrl(endpoint: string, apiKey?: string): string {
	if (!apiKey) return endpoint;
	return `${endpoint}?api_key=${encodeURIComponent(apiKey)}`;
}

export async function getWorkspaces(apiKey?: string): Promise<Team[]> {
	return get<Team[]>(buildUrl('/clickup/workspaces', apiKey));
}

export async function getSpaces(teamId: string, apiKey?: string): Promise<Space[]> {
	return get<Space[]>(buildUrl(`/clickup/workspaces/${teamId}/spaces`, apiKey));
}

export async function getFolders(spaceId: string, apiKey?: string): Promise<Folder[]> {
	return get<Folder[]>(buildUrl(`/clickup/spaces/${spaceId}/folders`, apiKey));
}

export async function getListsInFolder(folderId: string, apiKey?: string): Promise<List[]> {
	return get<List[]>(buildUrl(`/clickup/folders/${folderId}/lists`, apiKey));
}

export async function getFolderlessLists(spaceId: string, apiKey?: string): Promise<List[]> {
	return get<List[]>(buildUrl(`/clickup/spaces/${spaceId}/lists`, apiKey));
}

export async function getListStatuses(listId: string, apiKey?: string): Promise<Status[]> {
	return get<Status[]>(buildUrl(`/clickup/lists/${listId}/statuses`, apiKey));
}
