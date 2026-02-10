/**
 * Projects API client
 * Handles all API calls related to project management
 */

export interface Project {
  id: number;
  name: string;
  description?: string;
  status: 'active' | 'archived';
  repo_path: string;
  github_url?: string;
  dev_branch: string;
  clickup_api_key?: string;
  clickup_workspace_id?: string;
  clickup_space_id?: string;
  clickup_folder_id?: string;
  clickup_list_id?: string;
  agent_prompt?: string;
  agent_model: string;
  parallel_limit: number;
  created_at: string;
  updated_at: string;
}

export interface ProjectListItem {
  id: number;
  name: string;
  description?: string;
  status: string;
  repo_path: string;
  dev_branch: string;
  workflow_count: number;
  active_task_count: number;
}

export interface CreateProjectRequest {
  name: string;
  description?: string;
  repo_path: string;
  dev_branch?: string;
  clickup_api_key?: string;
  clickup_workspace_id?: string;
  clickup_space_id?: string;
  clickup_folder_id?: string;
  clickup_list_id?: string;
  agent_prompt?: string;
  agent_model?: string;
  parallel_limit?: number;
}

export interface UpdateProjectRequest {
  name?: string;
  description?: string;
  repo_path?: string;
  dev_branch?: string;
  clickup_api_key?: string;
  clickup_workspace_id?: string;
  clickup_space_id?: string;
  clickup_folder_id?: string;
  clickup_list_id?: string;
  agent_prompt?: string;
  agent_model?: string;
  parallel_limit?: number;
}

export interface CloneProjectRequest {
  name: string;
  description?: string;
  github_url: string;
  target_path: string;
  dev_branch?: string;
  clickup_api_key?: string;
  clickup_workspace_id?: string;
  clickup_space_id?: string;
  clickup_folder_id?: string;
  clickup_list_id?: string;
  agent_prompt?: string;
  agent_model?: string;
  parallel_limit?: number;
}

const API_BASE = '/api/projects';

/**
 * List all projects
 */
export async function listProjects(): Promise<ProjectListItem[]> {
  const response = await fetch(`${API_BASE}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  // Handle 404 as "no projects" rather than an error
  if (response.status === 404) {
    return [];
  }

  if (!response.ok) {
    throw new Error(`Failed to list projects: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Get a specific project by ID
 */
export async function getProject(id: number): Promise<Project> {
  const response = await fetch(`${API_BASE}/${id}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    if (response.status === 404) {
      throw new Error('Project not found');
    }
    throw new Error(`Failed to get project: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Create a new project
 */
export async function createProject(data: CreateProjectRequest): Promise<Project> {
  const response = await fetch(`${API_BASE}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to create project: ${error}`);
  }

  return response.json();
}

/**
 * Update a project
 */
export async function updateProject(id: number, data: UpdateProjectRequest): Promise<Project> {
  const response = await fetch(`${API_BASE}/${id}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to update project: ${error}`);
  }

  return response.json();
}

/**
 * Delete a project
 */
export async function deleteProject(id: number): Promise<void> {
  const response = await fetch(`${API_BASE}/${id}`, {
    method: 'DELETE',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to delete project: ${response.statusText}`);
  }
}

/**
 * Archive a project
 */
export async function archiveProject(id: number): Promise<Project> {
  const response = await fetch(`${API_BASE}/${id}/archive`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to archive project: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Clone a project from GitHub
 */
export async function cloneProjectFromGithub(data: CloneProjectRequest): Promise<Project> {
  const response = await fetch(`${API_BASE}/clone`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to clone project: ${error}`);
  }

  return response.json();
}

export interface FolderInfo {
  name: string;
  path: string;
  is_git_repo: boolean;
}

export interface FolderListResponse {
  current_path: string;
  base_path: string;
  folders: FolderInfo[];
  can_go_up: boolean;
}

export interface GitStatusResponse {
  is_git_repo: boolean;
  branch?: string;
  path: string;
}

export interface InitGitResponse {
  success: boolean;
  message: string;
  branch: string;
}

/**
 * List available folders for project creation
 */
export async function listFolders(path?: string): Promise<FolderListResponse> {
  const url = new URL(`${API_BASE}/folders`, window.location.origin);
  if (path) {
    url.searchParams.set('path', path);
  }

  const response = await fetch(url.toString(), {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to list folders: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Check git status of a folder
 */
export async function checkGitStatus(path: string): Promise<GitStatusResponse> {
  const response = await fetch(`${API_BASE}/git/check`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ path }),
  });

  if (!response.ok) {
    throw new Error(`Failed to check git status: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Initialize git repository in a folder
 */
export async function initializeGit(path: string): Promise<InitGitResponse> {
  const response = await fetch(`${API_BASE}/git/init`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ path }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to initialize git: ${error}`);
  }

  return response.json();
}
