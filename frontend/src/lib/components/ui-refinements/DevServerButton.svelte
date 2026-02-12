<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		detectDevServer,
		startDevServer,
		stopDevServer,
		getDevServerStatus
	} from '$lib/api/devserver';

	interface Props {
		repoPath: string;
	}

	let { repoPath }: Props = $props();

	type DevServerState = 'detecting' | 'not-found' | 'stopped' | 'starting' | 'running' | 'stopping';
	let serverState: DevServerState = $state('detecting');
	let packageJsonDir: string | null = $state(null);
	let packageManager: string | null = $state(null);
	let pollInterval: ReturnType<typeof setInterval> | null = $state(null);

	onMount(() => {
		detect();
	});

	onDestroy(() => {
		clearPoll();
	});

	$effect(() => {
		// Re-detect when repoPath changes
		if (repoPath) {
			detect();
		}
	});

	function clearPoll() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}

	async function detect() {
		if (!repoPath) {
			serverState = 'not-found';
			return;
		}
		serverState = 'detecting';
		try {
			const result = await detectDevServer(repoPath);
			if (result.found && result.has_dev_script) {
				packageJsonDir = result.package_json_dir;
				packageManager = result.package_manager;
				// Check if already running
				const status = await getDevServerStatus(repoPath);
				serverState = status.running ? 'running' : 'stopped';
				if (status.running) {
					startPoll();
				}
			} else {
				serverState = 'not-found';
			}
		} catch {
			serverState = 'not-found';
		}
	}

	function startPoll() {
		clearPoll();
		pollInterval = setInterval(async () => {
			try {
				const status = await getDevServerStatus(repoPath);
				if (!status.running && serverState === 'running') {
					serverState = 'stopped';
					clearPoll();
				}
			} catch {
				// ignore poll errors
			}
		}, 3000);
	}

	async function handleStart() {
		if (!packageJsonDir || !packageManager) return;
		serverState = 'starting';
		try {
			const result = await startDevServer(repoPath, packageJsonDir, packageManager);
			if (result.success) {
				serverState = 'running';
				startPoll();
			} else {
				serverState = 'stopped';
			}
		} catch {
			serverState = 'stopped';
		}
	}

	async function handleStop() {
		serverState = 'stopping';
		clearPoll();
		try {
			await stopDevServer(repoPath);
			serverState = 'stopped';
		} catch {
			serverState = 'running';
			startPoll();
		}
	}
</script>

{#if serverState !== 'not-found' && serverState !== 'detecting'}
	<button
		onclick={() => {
			if (serverState === 'stopped') handleStart();
			else if (serverState === 'running') handleStop();
		}}
		disabled={serverState === 'starting' || serverState === 'stopping'}
		class="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium transition-colors {serverState === 'running'
			? 'bg-green-100 text-green-700 hover:bg-green-200'
			: serverState === 'starting' || serverState === 'stopping'
				? 'bg-amber-100 text-amber-700 cursor-wait'
				: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
	>
		{#if serverState === 'stopped'}
			<!-- Play icon -->
			<svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
				<path d="M6.3 2.841A1.5 1.5 0 004 4.11V15.89a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
			</svg>
			<span>Start Dev</span>
		{:else if serverState === 'starting'}
			<!-- Spinner -->
			<svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
			</svg>
			<span>Starting...</span>
		{:else if serverState === 'running'}
			<!-- Stop icon -->
			<svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
				<path d="M5.25 3A2.25 2.25 0 003 5.25v9.5A2.25 2.25 0 005.25 17h9.5A2.25 2.25 0 0017 14.75v-9.5A2.25 2.25 0 0014.75 3h-9.5z" />
			</svg>
			<span>Stop Dev</span>
		{:else if serverState === 'stopping'}
			<!-- Spinner -->
			<svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
			</svg>
			<span>Stopping...</span>
		{/if}
	</button>
{/if}
