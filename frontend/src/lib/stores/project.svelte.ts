/**
 * Project store for managing current project state
 * Persists to localStorage for session persistence
 */

import { getProject, listProjects, type Project } from '$lib/api/projects';

const STORAGE_KEY = 'clickup-orchestrator-project-id';

interface ProjectStore {
  currentProjectId: number | null;
  project: Project | null;
  isLoading: boolean;
  error: string | null;
  setProjectId: (id: number) => void;
  loadProject: (id: number) => Promise<void>;
  clearProject: () => void;
  getAllProjects: () => Promise<Project[]>;
}

function createProjectStore(): ProjectStore {
  let currentProjectId = $state<number | null>(null);
  let project = $state<Project | null>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  // Initialize from localStorage on creation
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        currentProjectId = parseInt(stored, 10);
      } catch (e) {
        console.error('Failed to parse stored project ID:', e);
      }
    }
  }

  return {
    get currentProjectId() {
      return currentProjectId;
    },
    get project() {
      return project;
    },
    get isLoading() {
      return isLoading;
    },
    get error() {
      return error;
    },
    setProjectId: (id: number) => {
      currentProjectId = id;
      project = null;
      error = null;
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, id.toString());
      }
    },
    loadProject: async (id: number) => {
      isLoading = true;
      error = null;
      try {
        const data = await getProject(id);
        project = data;
        currentProjectId = id;
        if (typeof window !== 'undefined') {
          localStorage.setItem(STORAGE_KEY, id.toString());
        }
      } catch (err) {
        error = err instanceof Error ? err.message : 'Failed to load project';
        project = null;
      } finally {
        isLoading = false;
      }
    },
    clearProject: () => {
      currentProjectId = null;
      project = null;
      error = null;
      if (typeof window !== 'undefined') {
        localStorage.removeItem(STORAGE_KEY);
      }
    },
    getAllProjects: async () => {
      try {
        return await listProjects();
      } catch (err) {
        error = err instanceof Error ? err.message : 'Failed to load projects';
        return [];
      }
    },
  };
}

// Create a single instance
export const projectStore = createProjectStore();

/**
 * Alternative hook-based approach for Svelte components
 */
export function useCurrentProject() {
  return projectStore;
}
