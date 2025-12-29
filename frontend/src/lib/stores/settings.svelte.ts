// Settings store using Svelte 5 runes

import { getSettings, updateSettings } from '$lib/api/settings';

// Settings state
let settings = $state<Record<string, string>>({});
let loading = $state(false);
let error = $state<string | null>(null);
let dirty = $state(false);

// Default settings
const defaults: Record<string, string> = {
	parallel_limit: '1',
	trigger_status: 'Ready for Dev',
	target_status: 'In Development',
	target_repo_path: '',
	dev_branch: 'dev',
	clickup_workspace_id: '',
	clickup_space_id: '',
	clickup_folder_id: '',
	clickup_list_id: ''
};

export function useSettings() {
	async function load() {
		loading = true;
		error = null;
		try {
			settings = await getSettings();
			dirty = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load settings';
			settings = { ...defaults };
		} finally {
			loading = false;
		}
	}

	async function save() {
		loading = true;
		error = null;
		try {
			settings = await updateSettings(settings);
			dirty = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save settings';
		} finally {
			loading = false;
		}
	}

	function get(key: string): string {
		return settings[key] ?? defaults[key] ?? '';
	}

	function set(key: string, value: string) {
		settings[key] = value;
		dirty = true;
	}

	return {
		get settings() {
			return settings;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get dirty() {
			return dirty;
		},
		load,
		save,
		get,
		set
	};
}
