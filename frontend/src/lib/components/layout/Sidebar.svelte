<script lang="ts">
	import { page } from '$app/stores';
	import { useCurrentProject } from '$lib/stores/project.svelte';
	import { listProjects } from '$lib/api/projects';
	import ProjectSelector from '../projects/ProjectSelector.svelte';

	interface NavItem {
		href: string;
		label: string;
		icon: string;
		experimental?: boolean;
		section?: string;
	}

	interface Props {
		collapsed?: boolean;
	}

	let { collapsed = $bindable(false) }: Props = $props();
	const projectStore = useCurrentProject();
	let projects = $state<any[]>([]);
	let loadingProjects = $state(true);

	// Load projects on mount
	$effect.pre(() => {
		loadProjects();
	});

	async function loadProjects() {
		try {
			projects = await projectStore.getAllProjects();
		} catch (e) {
			console.error('Failed to load projects:', e);
		} finally {
			loadingProjects = false;
		}
	}

	const getNavItems = (): NavItem[] => {
		const projectId = projectStore.currentProjectId;
		const projectPath = projectId ? `/projects/${projectId}` : '';

		const items: NavItem[] = [
			{
				href: '/projects',
				label: 'All Projects',
				icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />`
			}
		];

		if (projectId) {
			// Core project navigation
			items.push(
				{
					href: `${projectPath}`,
					label: 'Dashboard',
					icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />`,
					section: 'project'
				},
				{
					href: `${projectPath}/tasks`,
					label: 'Tasks',
					icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />`,
					section: 'project'
				},
				{
					href: `${projectPath}/workflows`,
					label: 'Workflows',
					icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />`,
					section: 'project'
				},
				{
					href: `${projectPath}/settings`,
					label: 'Project Settings',
					icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />`,
					section: 'project'
				}
			);

			// Experimental tools (project-scoped)
			items.push(
				{
					href: '/voice',
					label: 'Voice Assistant',
					icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />`,
					experimental: true,
					section: 'tools'
				},
				{
					href: '/ui-refinements',
					label: 'UI Modifier',
					icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122" />`,
					experimental: true,
					section: 'tools'
				},
				{
					href: '/editor',
					label: 'File Editor',
					icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />`,
					experimental: true,
					section: 'tools'
				}
			);
		}

		// Global settings always at bottom
		items.push({
			href: '/settings',
			label: 'Global Settings',
			icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />`,
			section: 'global'
		});

		return items;
	};

	function isActive(href: string): boolean {
		const pathname = $page.url.pathname;
		if (href === '/projects') {
			return pathname === '/projects' || pathname === '/';
		}
		// Exact match for project dashboard to avoid matching sub-routes
		const projectId = projectStore.currentProjectId;
		if (projectId && href === `/projects/${projectId}`) {
			return pathname === href;
		}
		return pathname.startsWith(href);
	}

	function toggleCollapse() {
		collapsed = !collapsed;
	}

	// Group items by section for rendering with dividers
	function getGroupedItems() {
		const items = getNavItems();
		const groups: { section: string; items: NavItem[] }[] = [];
		let currentSection = '';

		for (const item of items) {
			const section = item.section || 'default';
			if (section !== currentSection) {
				groups.push({ section, items: [item] });
				currentSection = section;
			} else {
				groups[groups.length - 1].items.push(item);
			}
		}

		return groups;
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

	<!-- Project Selector -->
	<div class="px-2 py-3 border-b border-gray-800">
		{#if !collapsed && !loadingProjects}
			<div class="text-xs text-gray-400 px-1 mb-2">Current Project</div>
			<ProjectSelector {projects} />
		{:else if collapsed}
			<div class="flex justify-center" title="Select Project">
				<svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
				</svg>
			</div>
		{/if}
	</div>

	<!-- Navigation -->
	<nav class="flex-1 py-4 px-2 space-y-1 overflow-y-auto">
		{#each getGroupedItems() as group, groupIndex}
			{#if groupIndex > 0}
				<!-- Section divider -->
				<div class="py-2">
					{#if !collapsed}
						{#if group.section === 'project'}
							<div class="px-3 text-[10px] font-semibold uppercase tracking-wider text-gray-500">Project</div>
						{:else if group.section === 'tools'}
							<div class="px-3 text-[10px] font-semibold uppercase tracking-wider text-gray-500">Tools</div>
						{:else if group.section === 'global'}
							<div class="border-t border-gray-800 mt-2 pt-3 px-3 text-[10px] font-semibold uppercase tracking-wider text-gray-500">System</div>
						{/if}
					{:else}
						<div class="border-t border-gray-800 mx-2"></div>
					{/if}
				</div>
			{/if}

			{#each group.items as item}
				<a
					href={item.href}
					class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors
						{isActive(item.href)
						? 'bg-indigo-600 text-white'
						: 'text-gray-400 hover:bg-gray-800 hover:text-white'}
						{collapsed ? 'justify-center' : ''}"
					title={collapsed ? item.label + (item.experimental ? ' (Beta)' : '') : ''}
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
		{/each}

		{#if !projectStore.currentProjectId && !collapsed}
			<div class="mt-4 mx-2 rounded-lg border border-dashed border-gray-700 p-3 text-center">
				<p class="text-xs text-gray-500">Select a project to access tasks, workflows, and tools</p>
			</div>
		{/if}
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
