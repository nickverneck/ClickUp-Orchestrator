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

export async function getWorkspaces(): Promise<Team[]> {
	return get<Team[]>('/clickup/workspaces');
}

export async function getSpaces(teamId: string): Promise<Space[]> {
	return get<Space[]>(`/clickup/workspaces/${teamId}/spaces`);
}

export async function getFolders(spaceId: string): Promise<Folder[]> {
	return get<Folder[]>(`/clickup/spaces/${spaceId}/folders`);
}

export async function getListsInFolder(folderId: string): Promise<List[]> {
	return get<List[]>(`/clickup/folders/${folderId}/lists`);
}

export async function getFolderlessLists(spaceId: string): Promise<List[]> {
	return get<List[]>(`/clickup/spaces/${spaceId}/lists`);
}

export async function getListStatuses(listId: string): Promise<Status[]> {
	return get<Status[]>(`/clickup/lists/${listId}/statuses`);
}
