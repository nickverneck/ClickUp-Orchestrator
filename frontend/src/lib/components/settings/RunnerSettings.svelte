<script lang="ts">
	import { getListStatuses, type Status } from '$lib/api/clickup';

	interface Props {
		parallelLimit: number;
		triggerStatus: string;
		targetStatus: string;
		agentPrompt: string;
		baPrompt: string;
		listId: string | null;
		onParallelLimitChange: (limit: number) => void;
		onTriggerStatusChange: (status: string) => void;
		onTargetStatusChange: (status: string) => void;
		onAgentPromptChange: (prompt: string) => void;
		onBaPromptChange: (prompt: string) => void;
	}

	let {
		parallelLimit,
		triggerStatus,
		targetStatus,
		agentPrompt,
		baPrompt,
		listId,
		onParallelLimitChange,
		onTriggerStatusChange,
		onTargetStatusChange,
		onAgentPromptChange,
		onBaPromptChange
	}: Props = $props();

	let statuses = $state<Status[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	// Fetch statuses when listId changes
	$effect(() => {
		if (listId) {
			loadStatuses(listId);
		} else {
			statuses = [];
		}
	});

	async function loadStatuses(id: string) {
		loading = true;
		error = null;
		try {
			statuses = await getListStatuses(id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load statuses';
			statuses = [];
		} finally {
			loading = false;
		}
	}

	function handleParallelLimitChange(e: Event) {
		onParallelLimitChange(parseInt((e.target as HTMLInputElement).value, 10));
	}

	function handleTriggerStatusChange(e: Event) {
		onTriggerStatusChange((e.target as HTMLSelectElement).value);
	}

	function handleTargetStatusChange(e: Event) {
		onTargetStatusChange((e.target as HTMLSelectElement).value);
	}
</script>

<div class="space-y-4">
	<h3 class="text-lg font-medium text-gray-900">Task Runner Settings</h3>

	<div class="grid grid-cols-1 gap-4 lg:grid-cols-3">
		<!-- Parallel Limit -->
		<div>
			<label for="parallel-limit" class="block text-sm font-medium text-gray-700">
				Parallel Task Limit
			</label>
			<div class="mt-1">
				<div class="flex items-center gap-4">
					<input
						type="range"
						id="parallel-limit"
						min="1"
						max="5"
						value={parallelLimit}
						oninput={handleParallelLimitChange}
						class="h-2 w-full cursor-pointer appearance-none rounded-lg bg-gray-200"
					/>
					<span class="w-8 text-center text-lg font-semibold text-gray-900">{parallelLimit}</span>
				</div>
			</div>
			<p class="mt-1 text-sm text-gray-500">
				Maximum number of tasks to run simultaneously
			</p>
		</div>

		<!-- Trigger Status -->
		<div>
			<label for="trigger-status" class="block text-sm font-medium text-gray-700">
				Trigger Status
			</label>
			{#if statuses.length > 0}
				<select
					id="trigger-status"
					value={triggerStatus}
					onchange={handleTriggerStatusChange}
					disabled={loading}
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
				>
					<option value="">Select status...</option>
					{#each statuses as status}
						<option value={status.status}>
							{status.status}
						</option>
					{/each}
				</select>
			{:else}
				<input
					type="text"
					id="trigger-status"
					value={triggerStatus}
					oninput={(e) => onTriggerStatusChange((e.target as HTMLInputElement).value)}
					placeholder="Ready for Dev"
					disabled={loading}
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
				/>
			{/if}
			<p class="mt-1 text-sm text-gray-500">
				{#if !listId}
					Select a ClickUp list first to see available statuses
				{:else if loading}
					Loading statuses...
				{:else if error}
					<span class="text-red-600">{error}</span>
				{:else}
					Status that triggers task pickup from ClickUp
				{/if}
			</p>
		</div>

		<!-- Target Status -->
		<div>
			<label for="target-status" class="block text-sm font-medium text-gray-700">
				Target Status
			</label>
			{#if statuses.length > 0}
				<select
					id="target-status"
					value={targetStatus}
					onchange={handleTargetStatusChange}
					disabled={loading}
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
				>
					<option value="">Select status...</option>
					{#each statuses as status}
						<option value={status.status}>
							{status.status}
						</option>
					{/each}
				</select>
			{:else}
				<input
					type="text"
					id="target-status"
					value={targetStatus}
					oninput={(e) => onTargetStatusChange((e.target as HTMLInputElement).value)}
					placeholder="In Development"
					disabled={loading}
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm disabled:bg-gray-100"
				/>
			{/if}
			<p class="mt-1 text-sm text-gray-500">
				{#if !listId}
					Select a ClickUp list first to see available statuses
				{:else if loading}
					Loading statuses...
				{:else}
					Status to set when a task is picked up
				{/if}
			</p>
		</div>
	</div>

	<!-- Agent Prompt -->
	<div>
		<label for="agent-prompt" class="block text-sm font-medium text-gray-700">
			Coding Agent Prompt (Global Instructions)
		</label>
		<div class="mt-1">
			<textarea
				id="agent-prompt"
				rows="4"
				value={agentPrompt}
				oninput={(e) => onAgentPromptChange((e.target as HTMLTextAreaElement).value)}
				placeholder="Enter global instructions that will be combined with each task's description..."
				class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
			></textarea>
		</div>
		<p class="mt-1 text-sm text-gray-500">
			These instructions will be appended to every task's description when spawning a coding agent.
		</p>
	</div>

	<!-- Business Analyst Prompt (Experimental) -->
	<div class="rounded-lg border-2 border-dashed border-amber-300 bg-amber-50/50 p-4">
		<div class="flex items-center gap-2 mb-3">
			<label for="ba-prompt" class="block text-sm font-medium text-gray-700">
				Business Analyst Prompt
			</label>
			<span class="text-[10px] font-medium bg-amber-500/20 text-amber-700 px-1.5 py-0.5 rounded">
				EXPERIMENTAL
			</span>
		</div>
		<div class="mt-1">
			<textarea
				id="ba-prompt"
				rows="4"
				value={baPrompt}
				oninput={(e) => onBaPromptChange((e.target as HTMLTextAreaElement).value)}
				placeholder="Instructions for the Business Analyst agent that converts voice recordings into actionable tasks..."
				class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
			></textarea>
		</div>
		<p class="mt-2 text-sm text-gray-500">
			Used by the Voice Assistant to convert spoken requirements and screen context into structured tasks
			that coding agents can work on.
		</p>
	</div>

	<!-- Info Box -->
	<div class="rounded-md bg-blue-50 p-4">
		<div class="flex">
			<div class="flex-shrink-0">
				<svg class="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
					<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
				</svg>
			</div>
			<div class="ml-3">
				<h3 class="text-sm font-medium text-blue-800">How it works</h3>
				<div class="mt-2 text-sm text-blue-700">
					<p>
						The orchestrator polls ClickUp every 30 seconds for tasks with the <strong>Trigger Status</strong>.
						When a task is picked up, it's moved to the <strong>Target Status</strong> and a CLI agent is spawned
						to work on it. Tasks are picked by priority (Urgent &gt; High &gt; Normal &gt; Low).
					</p>
				</div>
			</div>
		</div>
	</div>
</div>
