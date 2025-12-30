import { get, post, put, del } from './client';
import type { FileNode, FileContent } from '$lib/types/editor';

export async function getFileTree(path: string): Promise<FileNode[]> {
	return get<FileNode[]>(`/files/tree?path=${encodeURIComponent(path)}`);
}

export async function getFileContent(path: string): Promise<FileContent> {
	return get<FileContent>(`/files/content?path=${encodeURIComponent(path)}`);
}

export async function saveFile(path: string, content: string): Promise<{ success: boolean }> {
	return put<{ success: boolean }>('/files/content', { path, content });
}

export async function createFile(
	path: string,
	isDirectory: boolean
): Promise<{ success: boolean }> {
	return post<{ success: boolean }>('/files/create', { path, is_directory: isDirectory });
}

export async function deleteFile(path: string): Promise<{ success: boolean }> {
	return del<{ success: boolean }>(`/files?path=${encodeURIComponent(path)}`);
}

export async function renameFile(
	oldPath: string,
	newPath: string
): Promise<{ success: boolean }> {
	return post<{ success: boolean }>('/files/rename', { old_path: oldPath, new_path: newPath });
}
