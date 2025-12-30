<script lang="ts">
	import type { AgentType } from '$lib/types/ui-refinements';

	interface Props {
		selected: AgentType;
		onChange: (agent: AgentType) => void;
	}

	let { selected, onChange }: Props = $props();

	const agents: { id: AgentType; label: string; icon: string }[] = [
		{
			id: 'claude',
			label: 'Claude',
			icon: '🟣'
		},
		{
			id: 'codex',
			label: 'Codex',
			icon: '🟢'
		},
		{
			id: 'gemini',
			label: 'Gemini',
			icon: '🔵'
		}
	];

	let isOpen = $state(false);

	function handleSelect(agent: AgentType) {
		onChange(agent);
		isOpen = false;
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.agent-selector')) {
			isOpen = false;
		}
	}
</script>

<svelte:window on:click={handleClickOutside} />

<div class="relative agent-selector">
	<button
		onclick={() => (isOpen = !isOpen)}
		class="flex items-center gap-2 px-3 py-1.5 bg-white border border-gray-300 rounded-lg hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent text-sm"
	>
		<span>{agents.find((a) => a.id === selected)?.icon}</span>
		<span class="font-medium text-gray-700">{agents.find((a) => a.id === selected)?.label}</span>
		<svg
			class="w-4 h-4 text-gray-400 transition-transform {isOpen ? 'rotate-180' : ''}"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>

	{#if isOpen}
		<div class="absolute right-0 mt-1 w-40 bg-white border border-gray-200 rounded-lg shadow-lg z-10 py-1">
			{#each agents as agent}
				<button
					onclick={() => handleSelect(agent.id)}
					class="w-full flex items-center gap-2 px-3 py-2 hover:bg-gray-50 text-left {selected === agent.id
						? 'bg-indigo-50'
						: ''}"
				>
					<span>{agent.icon}</span>
					<span class="text-sm font-medium text-gray-700">{agent.label}</span>
					{#if selected === agent.id}
						<svg class="w-4 h-4 text-indigo-600 ml-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
						</svg>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>
