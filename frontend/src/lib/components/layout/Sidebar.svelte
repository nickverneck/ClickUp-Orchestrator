<script lang="ts">
	import { page } from '$app/stores';

	interface Props {
		collapsed?: boolean;
	}

	let { collapsed = $bindable(false) }: Props = $props();

	const navItems = [
		{
			href: '/',
			label: 'Dashboard',
			icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />`
		},
		{
			href: '/voice',
			label: 'Voice Assistant',
			icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />`,
			experimental: true
		},
		{
			href: '/ui-refinements',
			label: 'UI Modifier',
			icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122" />`,
			experimental: true
		},
		{
			href: '/editor',
			label: 'File Editor',
			icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />`,
			experimental: true
		},
		{
			href: '/settings',
			label: 'Settings',
			icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />`
		}
	];

	function isActive(href: string): boolean {
		if (href === '/') {
			return $page.url.pathname === '/' || $page.url.pathname.startsWith('/task/');
		}
		return $page.url.pathname.startsWith(href);
	}

	function toggleCollapse() {
		collapsed = !collapsed;
	}
</script>

<aside
	class="flex flex-col bg-gray-900 text-white transition-all duration-300 ease-in-out {collapsed
		? 'w-16'
		: 'w-64'}"
>
	<!-- Header -->
	<div class="flex h-16 items-center justify-between px-4 border-b border-gray-800">
		{#if !collapsed}
			<span class="text-lg font-bold text-indigo-400 truncate">ClickUp Orchestrator</span>
		{/if}
		<button
			onclick={toggleCollapse}
			class="p-2 rounded-lg hover:bg-gray-800 transition-colors {collapsed ? 'mx-auto' : ''}"
			title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
		>
			<svg
				class="h-5 w-5 text-gray-400 transition-transform duration-300 {collapsed
					? 'rotate-180'
					: ''}"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
				/>
			</svg>
		</button>
	</div>

	<!-- Navigation -->
	<nav class="flex-1 py-4 px-2 space-y-1">
		{#each navItems as item}
			<a
				href={item.href}
				class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors
					{isActive(item.href)
					? 'bg-indigo-600 text-white'
					: 'text-gray-400 hover:bg-gray-800 hover:text-white'}
					{collapsed ? 'justify-center' : ''}"
				title={collapsed ? item.label + (item.experimental ? ' (Experimental)' : '') : ''}
			>
				<div class="relative flex-shrink-0">
					<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						{@html item.icon}
					</svg>
					{#if item.experimental && collapsed}
						<span class="absolute -top-1 -right-1 w-2 h-2 bg-amber-500 rounded-full"></span>
					{/if}
				</div>
				{#if !collapsed}
					<span class="truncate">{item.label}</span>
					{#if item.experimental}
						<span class="ml-auto text-[10px] font-medium bg-amber-500/20 text-amber-400 px-1.5 py-0.5 rounded">
							BETA
						</span>
					{/if}
				{/if}
			</a>
		{/each}
	</nav>

	<!-- Footer -->
	<div class="p-4 border-t border-gray-800">
		{#if !collapsed}
			<div class="flex items-center gap-2 text-xs text-gray-500">
				<div class="w-2 h-2 rounded-full bg-green-500"></div>
				<span>Backend Connected</span>
			</div>
		{:else}
			<div class="flex justify-center" title="Backend Connected">
				<div class="w-2 h-2 rounded-full bg-green-500"></div>
			</div>
		{/if}
	</div>
</aside>
