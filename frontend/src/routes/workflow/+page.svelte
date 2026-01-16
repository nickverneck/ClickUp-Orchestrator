<script lang="ts">
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import {
		getWorkflow,
		pauseWorkflow,
		startWorkflow,
		updateWorkflow,
		type WorkflowAction,
		type WorkflowConfig,
		type WorkflowEdge,
		type WorkflowNode,
		type WorkflowStatus
	} from '$lib/api/workflow';

	type QueueSettings = {
		provider?: string;
		baseBranch?: string;
		clickup?: {
			listId?: string;
			triggerStatus?: string;
		};
	};

	type AgentSettings = {
		systemPrompt?: string;
		model?: string;
		clickupStatus?: string;
		capacity?: number;
	};

	type BucketSettings = {
		bucket?: string;
	};

	const NODE_WIDTH = 220;
	const NODE_HEIGHT = 120;

	let workflow = $state<WorkflowConfig | null>(null);
	let status = $state<WorkflowStatus>('paused');
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);
	let dirty = $state(false);
	let sidebarCollapsed = $state(false);

	let selectedNodeId = $state<string | null>(null);
	let selectedEdgeId = $state<string | null>(null);

	let newEdgeSource = $state('');
	let newEdgeTarget = $state('');
	let newEdgeOutput = $state<'default' | 'success' | 'error'>('default');

	let canvasEl = $state<HTMLDivElement | null>(null);
	let draggingNodeId: string | null = null;
	let dragOffset = { x: 0, y: 0 };

	onMount(() => {
		loadWorkflow();
		const saved = localStorage.getItem('sidebarCollapsed');
		if (saved !== null) {
			sidebarCollapsed = saved === 'true';
		}

		const handleMove = (event: PointerEvent) => {
			if (!draggingNodeId || !workflow || !canvasEl) return;
			const rect = canvasEl.getBoundingClientRect();
			const x = event.clientX - rect.left - dragOffset.x;
			const y = event.clientY - rect.top - dragOffset.y;
			updateNodePosition(draggingNodeId, x, y);
		};

		const handleUp = () => {
			draggingNodeId = null;
		};

		window.addEventListener('pointermove', handleMove);
		window.addEventListener('pointerup', handleUp);

		return () => {
			window.removeEventListener('pointermove', handleMove);
			window.removeEventListener('pointerup', handleUp);
		};
	});

	$effect(() => {
		localStorage.setItem('sidebarCollapsed', String(sidebarCollapsed));
	});

	function selectedNode(): WorkflowNode | null {
		return workflow?.nodes.find((node) => node.id === selectedNodeId) ?? null;
	}

	function selectedEdge(): WorkflowEdge | null {
		return workflow?.edges.find((edge) => edge.id === selectedEdgeId) ?? null;
	}

	function nodeFieldId(field: string) {
		return `node-${selectedNodeId ?? 'none'}-${field}`;
	}

	function edgeFieldId(field: string) {
		return `edge-${selectedEdgeId ?? 'none'}-${field}`;
	}

	function actionFieldId(actionId: string, field: string) {
		return `action-${actionId}-${field}`;
	}

	function canEdit(): boolean {
		return status === 'paused';
	}

	async function loadWorkflow() {
		loading = true;
		error = null;
		try {
			const response = await getWorkflow();
			workflow = response.config;
			status = response.status;
			dirty = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load workflow';
		} finally {
			loading = false;
		}
	}

	function markDirty(next: WorkflowConfig) {
		workflow = next;
		dirty = true;
	}

	function updateNodePosition(id: string, x: number, y: number) {
		if (!workflow || !canEdit()) return;
		const nodes = workflow.nodes.map((node) =>
			node.id === id
				? {
						...node,
						position: {
							x: Math.max(0, Math.round(x)),
							y: Math.max(0, Math.round(y))
						}
					}
				: node
		);
		markDirty({ ...workflow, nodes });
	}

	function startDrag(event: PointerEvent, node: WorkflowNode) {
		if (!canEdit() || !canvasEl) return;
		selectedNodeId = node.id;
		selectedEdgeId = null;
		const rect = canvasEl.getBoundingClientRect();
		draggingNodeId = node.id;
		dragOffset = {
			x: event.clientX - rect.left - node.position.x,
			y: event.clientY - rect.top - node.position.y
		};
	}

	function selectNodeById(id: string) {
		selectedNodeId = id;
		selectedEdgeId = null;
	}

	function selectEdgeById(id: string) {
		selectedEdgeId = id;
		selectedNodeId = null;
	}

	function addNode(type: 'queue' | 'agent' | 'bucket') {
		if (!workflow || !canEdit()) return;
		const index = workflow.nodes.length;
		const baseX = 140 + (index % 3) * 260;
		const baseY = 120 + Math.floor(index / 3) * 180;
		const id = `node-${type}-${Date.now().toString(36)}-${index}`;
		const label = type === 'agent' ? 'Agent' : type === 'queue' ? 'Queue' : 'Bucket';
		const settings =
			type === 'queue'
				? ({
						provider: 'clickup',
						baseBranch: 'dev',
						clickup: { listId: '', triggerStatus: 'Ready for Dev' }
					} as QueueSettings)
				: type === 'agent'
						? ({
								systemPrompt: '',
								model: 'codex',
								clickupStatus: 'In Development',
								capacity: 1
							} as AgentSettings)
						: ({ bucket: 'custom' } as BucketSettings);

		const node: WorkflowNode = {
			id,
			type,
			label,
			position: { x: baseX, y: baseY },
			settings: settings as Record<string, unknown>
		};

		markDirty({ ...workflow, nodes: [...workflow.nodes, node] });
		selectNodeById(id);
	}

	function deleteSelectedNode() {
		if (!workflow || !selectedNodeId || !canEdit()) return;
		const nodes = workflow.nodes.filter((node) => node.id !== selectedNodeId);
		const edges = workflow.edges.filter(
			(edge) => edge.source !== selectedNodeId && edge.target !== selectedNodeId
		);
		markDirty({ ...workflow, nodes, edges });
		selectedNodeId = null;
	}

	function updateNodeLabel(id: string, value: string) {
		if (!workflow || !canEdit()) return;
		const nodes = workflow.nodes.map((node) =>
			node.id === id ? { ...node, label: value } : node
		);
		markDirty({ ...workflow, nodes });
	}

	function updateNodeSettings(id: string, settings: Record<string, unknown>) {
		if (!workflow || !canEdit()) return;
		const nodes = workflow.nodes.map((node) =>
			node.id === id ? { ...node, settings } : node
		);
		markDirty({ ...workflow, nodes });
	}

	function addEdge() {
		if (!workflow || !canEdit()) return;
		if (!newEdgeSource || !newEdgeTarget || newEdgeSource === newEdgeTarget) {
			error = 'Select a valid source and target node.';
			return;
		}
		error = null;
		const id = `edge-${Date.now().toString(36)}-${workflow.edges.length}`;
		const edge: WorkflowEdge = {
			id,
			source: newEdgeSource,
			target: newEdgeTarget,
			output: newEdgeOutput === 'default' ? null : newEdgeOutput,
			actions: []
		};
		markDirty({ ...workflow, edges: [...workflow.edges, edge] });
		selectEdgeById(id);
	}

	function deleteSelectedEdge() {
		if (!workflow || !selectedEdgeId || !canEdit()) return;
		const edges = workflow.edges.filter((edge) => edge.id !== selectedEdgeId);
		markDirty({ ...workflow, edges });
		selectedEdgeId = null;
	}

	function updateEdgeOutput(edgeId: string, output: 'default' | 'success' | 'error') {
		if (!workflow || !canEdit()) return;
		const edges = workflow.edges.map((edge) =>
			edge.id === edgeId
				? { ...edge, output: output === 'default' ? null : output }
				: edge
		);
		markDirty({ ...workflow, edges });
	}

	function addAction(edgeId: string) {
		if (!workflow || !canEdit()) return;
		const action: WorkflowAction = {
			id: `action-${Date.now().toString(36)}-${Math.floor(Math.random() * 1000)}`,
			type: 'create_branch',
			settings: { branchPrefix: 'task/' }
		};
		const edges = workflow.edges.map((edge) =>
			edge.id === edgeId ? { ...edge, actions: [...edge.actions, action] } : edge
		);
		markDirty({ ...workflow, edges });
	}

	function removeAction(edgeId: string, actionId: string) {
		if (!workflow || !canEdit()) return;
		const edges = workflow.edges.map((edge) =>
			edge.id === edgeId
				? { ...edge, actions: edge.actions.filter((action) => action.id !== actionId) }
				: edge
		);
		markDirty({ ...workflow, edges });
	}

	function updateActionType(edgeId: string, actionId: string, type: string) {
		if (!workflow || !canEdit()) return;
		const edges = workflow.edges.map((edge) => {
			if (edge.id !== edgeId) return edge;
			return {
				...edge,
				actions: edge.actions.map((action) => {
					if (action.id !== actionId) return action;
					const settings =
						type === 'create_branch'
							? { branchPrefix: 'task/' }
							: type === 'update_clickup_status'
									? { status: '' }
									: { notes: '' };
					return { ...action, type, settings };
				})
			};
		});
		markDirty({ ...workflow, edges });
	}

	function updateActionSetting(
		edgeId: string,
		actionId: string,
		key: string,
		value: string
	) {
		if (!workflow || !canEdit()) return;
		const edges = workflow.edges.map((edge) => {
			if (edge.id !== edgeId) return edge;
			return {
				...edge,
				actions: edge.actions.map((action) =>
					action.id === actionId
						? { ...action, settings: { ...action.settings, [key]: value } }
						: action
				)
			};
		});
		markDirty({ ...workflow, edges });
	}

	function getNodeById(id: string): WorkflowNode | undefined {
		return workflow?.nodes.find((node) => node.id === id);
	}

	function edgePath(edge: WorkflowEdge): { d: string; color: string } | null {
		if (!workflow) return null;
		const source = getNodeById(edge.source);
		const target = getNodeById(edge.target);
		if (!source || !target) return null;

		const output = edge.output ?? 'default';
		const start = outputAnchor(source, output);
		const end = { x: target.position.x, y: target.position.y + NODE_HEIGHT / 2 };
		const distance = Math.max(120, Math.abs(end.x - start.x) / 2);
		const d = `M ${start.x} ${start.y} C ${start.x + distance} ${start.y}, ${end.x - distance} ${end.y}, ${end.x} ${end.y}`;
		const color = output === 'success' ? '#16a34a' : output === 'error' ? '#dc2626' : '#64748b';
		return { d, color };
	}

	function outputAnchor(node: WorkflowNode, output: string) {
		if (node.type === 'agent') {
			if (output === 'success') {
				return { x: node.position.x + NODE_WIDTH, y: node.position.y + 30 };
			}
			if (output === 'error') {
				return { x: node.position.x + NODE_WIDTH, y: node.position.y + NODE_HEIGHT - 30 };
			}
		}
		return { x: node.position.x + NODE_WIDTH, y: node.position.y + NODE_HEIGHT / 2 };
	}

	async function handleSave() {
		if (!workflow) return;
		saving = true;
		error = null;
		try {
			const response = await updateWorkflow(workflow);
			workflow = response.config;
			status = response.status;
			dirty = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save workflow';
		} finally {
			saving = false;
		}
	}

	async function toggleWorkflow() {
		saving = true;
		error = null;
		try {
			const response = status === 'paused' ? await startWorkflow() : await pauseWorkflow();
			status = response.status;
			workflow = response.config;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update workflow status';
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>Workflow Editor - ClickUp Orchestrator</title>
</svelte:head>

<div class="flex h-screen bg-gray-50">
	<Sidebar bind:collapsed={sidebarCollapsed} />

	<main class="flex-1 overflow-hidden">
		<div class="flex h-full flex-col">
			<div class="border-b border-gray-200 bg-white px-6 py-4">
				<div class="flex flex-wrap items-center justify-between gap-4">
					<div>
						<h1 class="text-2xl font-semibold text-gray-900">Workflow Editor</h1>
						<p class="text-sm text-gray-500">
							Design your pipeline with queue, agent, and bucket nodes.
						</p>
					</div>
					<div class="flex flex-wrap items-center gap-3">
						<span
							class="inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-semibold uppercase tracking-wide {status === 'running'
								? 'bg-emerald-100 text-emerald-700'
								: 'bg-amber-100 text-amber-700'}"
						>
							<span
								class="h-2 w-2 rounded-full {status === 'running'
									? 'bg-emerald-500'
									: 'bg-amber-500'}"
							></span>
							{status}
						</span>
						{#if dirty}
							<span class="text-sm text-amber-600">Unsaved changes</span>
						{/if}
						<button
							onclick={toggleWorkflow}
							disabled={saving || loading}
							class="inline-flex items-center rounded-md border border-gray-200 px-3 py-2 text-sm font-semibold text-gray-700 shadow-sm hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
						>
							{status === 'paused' ? 'Start Workflow' : 'Pause Workflow'}
						</button>
						<button
							onclick={handleSave}
							disabled={!dirty || saving || loading || !canEdit()}
							class="inline-flex items-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-50"
						>
							{saving ? 'Saving...' : 'Save Workflow'}
						</button>
					</div>
				</div>
			</div>

			{#if loading}
				<div class="flex flex-1 items-center justify-center">
					<svg class="h-8 w-8 animate-spin text-indigo-600" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
				</div>
			{:else}
				<div class="flex flex-1 flex-col overflow-hidden lg:flex-row">
					<div class="w-full flex-shrink-0 border-b border-gray-200 bg-white p-4 lg:h-full lg:w-64 lg:border-b-0 lg:border-r">
						<h2 class="text-sm font-semibold text-gray-700">Add Nodes</h2>
						<div class="mt-3 space-y-2">
							<button
								onclick={() => addNode('queue')}
								disabled={!canEdit()}
								class="w-full rounded-md border border-gray-200 px-3 py-2 text-left text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
							>
								+ Queue Node
							</button>
							<button
								onclick={() => addNode('agent')}
								disabled={!canEdit()}
								class="w-full rounded-md border border-gray-200 px-3 py-2 text-left text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
							>
								+ Agent Node
							</button>
							<button
								onclick={() => addNode('bucket')}
								disabled={!canEdit()}
								class="w-full rounded-md border border-gray-200 px-3 py-2 text-left text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
							>
								+ Bucket Node
							</button>
						</div>

						<div class="mt-6">
							<h2 class="text-sm font-semibold text-gray-700">Connections</h2>
							<div class="mt-3 space-y-3">
								<select
									bind:value={newEdgeSource}
									class="w-full rounded-md border border-gray-200 text-sm"
									disabled={!canEdit()}
								>
									<option value="">Source node</option>
									{#if workflow}
										{#each workflow.nodes as node}
											<option value={node.id}>{node.label}</option>
										{/each}
									{/if}
								</select>
								<select
									bind:value={newEdgeTarget}
									class="w-full rounded-md border border-gray-200 text-sm"
									disabled={!canEdit()}
								>
									<option value="">Target node</option>
									{#if workflow}
										{#each workflow.nodes as node}
											<option value={node.id}>{node.label}</option>
										{/each}
									{/if}
								</select>
								<select
									bind:value={newEdgeOutput}
									class="w-full rounded-md border border-gray-200 text-sm"
									disabled={!canEdit()}
								>
									<option value="default">Default output</option>
									<option value="success">Success (0)</option>
									<option value="error">Error (1)</option>
								</select>
								<button
									onclick={addEdge}
									disabled={!canEdit()}
									class="w-full rounded-md bg-gray-900 px-3 py-2 text-sm font-semibold text-white hover:bg-gray-800 disabled:cursor-not-allowed disabled:opacity-50"
								>
									Add Connection
								</button>
							</div>
							<div class="mt-4 space-y-2 text-sm text-gray-600">
								{#if workflow}
									{#each workflow.edges as edge}
										<button
											onclick={() => selectEdgeById(edge.id)}
											class="w-full rounded-md border border-gray-200 px-3 py-2 text-left hover:bg-gray-50 {edge.id === selectedEdgeId ? 'border-indigo-300 bg-indigo-50 text-indigo-700' : ''}"
										>
											{getNodeById(edge.source)?.label} ->
											{getNodeById(edge.target)?.label}
											{#if edge.output}
												<span class="ml-1 text-xs uppercase text-gray-500">({edge.output})</span>
											{/if}
										</button>
									{/each}
								{/if}
							</div>
						</div>
					</div>

					<div class="relative flex-1 overflow-auto bg-gray-100">
						<div
							class="relative h-full min-h-[640px] min-w-[900px]"
							bind:this={canvasEl}
							style="background-image: radial-gradient(rgba(148, 163, 184, 0.35) 1px, transparent 1px); background-size: 24px 24px;"
						>
							<svg class="absolute inset-0 h-full w-full">
								<defs>
									<marker
										id="arrow"
										markerWidth="10"
										markerHeight="10"
										refX="9"
										refY="5"
										orient="auto"
									>
										<path d="M0,0 L10,5 L0,10 z" fill="currentColor" />
									</marker>
								</defs>
								{#if workflow}
									{#each workflow.edges as edge}
										{#if edgePath(edge)}
											{@const path = edgePath(edge)}
											<path
												d={path?.d}
												stroke={path?.color}
												color={path?.color}
												stroke-width="2.5"
												fill="none"
												marker-end="url(#arrow)"
												onpointerdown={() => selectEdgeById(edge.id)}
												class="cursor-pointer {edge.id === selectedEdgeId ? 'drop-shadow-[0_0_6px_rgba(59,130,246,0.4)]' : ''}"
											/>
										{/if}
									{/each}
								{/if}
							</svg>

							{#if workflow}
								{#each workflow.nodes as node}
									<div
										class="absolute w-[220px] rounded-lg border bg-white shadow-sm {node.id === selectedNodeId
											? 'border-indigo-500 ring-2 ring-indigo-200'
											: 'border-gray-200'}"
										style={`left: ${node.position.x}px; top: ${node.position.y}px;`}
										role="button"
										tabindex="0"
										onpointerdown={(event) => startDrag(event, node)}
										onclick={() => selectNodeById(node.id)}
										onkeydown={(event) => {
											if (event.key === 'Enter' || event.key === ' ') {
												event.preventDefault();
												selectNodeById(node.id);
											}
										}}
									>
										<div class="flex items-center justify-between border-b border-gray-100 px-3 py-2 text-sm font-semibold text-gray-800">
											<span class="truncate">{node.label}</span>
											<span class="rounded-full bg-gray-100 px-2 py-0.5 text-[10px] uppercase text-gray-600">
												{node.type}
											</span>
										</div>
										<div class="px-3 py-2 text-xs text-gray-500">
											{#if node.type === 'queue'}
												Queue source: {(node.settings as QueueSettings).provider || 'clickup'}
											{:else if node.type === 'agent'}
												Model: {(node.settings as AgentSettings).model || 'codex'}
											{:else}
												Bucket: {(node.settings as BucketSettings).bucket || 'custom'}
											{/if}
										</div>

										<div class="pointer-events-none absolute -left-2 top-1/2 h-3 w-3 -translate-y-1/2 rounded-full border-2 border-white bg-gray-300"></div>
										{#if node.type === 'agent'}
											<div class="pointer-events-none absolute -right-2 top-[26px] h-3 w-3 rounded-full border-2 border-white bg-emerald-500"></div>
											<div class="pointer-events-none absolute -right-2 bottom-[26px] h-3 w-3 rounded-full border-2 border-white bg-rose-500"></div>
										{:else}
											<div class="pointer-events-none absolute -right-2 top-1/2 h-3 w-3 -translate-y-1/2 rounded-full border-2 border-white bg-slate-400"></div>
										{/if}
									</div>
								{/each}
							{/if}

							{#if !canEdit()}
								<div class="absolute inset-0 flex items-center justify-center bg-white/60 text-sm font-semibold text-gray-700">
									Pause workflow to edit nodes and connections.
								</div>
							{/if}
						</div>
					</div>

					<div class="w-full flex-shrink-0 border-t border-gray-200 bg-white p-4 lg:h-full lg:w-80 lg:border-l lg:border-t-0">
						<h2 class="text-sm font-semibold text-gray-700">Inspector</h2>
						{#if selectedNode()}
							<div class="mt-4 space-y-4 text-sm">
								<div>
									<label for={nodeFieldId('label')} class="text-xs font-semibold uppercase text-gray-500">
										Label
									</label>
									<input
										id={nodeFieldId('label')}
										class="mt-1 w-full rounded-md border border-gray-200 text-sm"
										value={selectedNode()?.label}
										oninput={(event) => updateNodeLabel(selectedNodeId as string, event.currentTarget.value)}
										disabled={!canEdit()}
									/>
								</div>

								{#if selectedNode()?.type === 'queue'}
									{@const settings = selectedNode()?.settings as QueueSettings}
									<div>
										<label for={nodeFieldId('provider')} class="text-xs font-semibold uppercase text-gray-500">
											Provider
										</label>
										<select
											id={nodeFieldId('provider')}
											class="mt-1 w-full rounded-md border border-gray-200 text-sm"
											value={settings?.provider || 'clickup'}
											onchange={(event) =>
												updateNodeSettings(selectedNodeId as string, {
													...settings,
													provider: event.currentTarget.value
												})}
											disabled={!canEdit()}
										>
											<option value="clickup">ClickUp</option>
											<option value="jira">Jira</option>
											<option value="custom">Custom</option>
										</select>
									</div>
									<div>
										<label
											for={nodeFieldId('base-branch')}
											class="text-xs font-semibold uppercase text-gray-500"
										>
											Base Branch
										</label>
										<input
											id={nodeFieldId('base-branch')}
											class="mt-1 w-full rounded-md border border-gray-200 text-sm"
											value={settings?.baseBranch || ''}
											oninput={(event) =>
												updateNodeSettings(selectedNodeId as string, {
													...settings,
													baseBranch: event.currentTarget.value
												})}
											disabled={!canEdit()}
										/>
									</div>
									<div>
										<label
											for={nodeFieldId('clickup-list')}
											class="text-xs font-semibold uppercase text-gray-500"
										>
											ClickUp List
										</label>
										<input
											id={nodeFieldId('clickup-list')}
											class="mt-1 w-full rounded-md border border-gray-200 text-sm"
											value={settings?.clickup?.listId || ''}
											oninput={(event) =>
												updateNodeSettings(selectedNodeId as string, {
													...settings,
													clickup: {
														...settings?.clickup,
														listId: event.currentTarget.value
													}
												})}
											disabled={!canEdit()}
										/>
									</div>
									<div>
										<label
											for={nodeFieldId('trigger-status')}
											class="text-xs font-semibold uppercase text-gray-500"
										>
											Trigger Status
										</label>
										<input
											id={nodeFieldId('trigger-status')}
											class="mt-1 w-full rounded-md border border-gray-200 text-sm"
											value={settings?.clickup?.triggerStatus || ''}
											oninput={(event) =>
												updateNodeSettings(selectedNodeId as string, {
													...settings,
													clickup: {
														...settings?.clickup,
														triggerStatus: event.currentTarget.value
													}
												})}
											disabled={!canEdit()}
										/>
									</div>
								{:else if selectedNode()?.type === 'agent'}
									{@const settings = selectedNode()?.settings as AgentSettings}
									<div>
										<label for={nodeFieldId('model')} class="text-xs font-semibold uppercase text-gray-500">
											Model
										</label>
										<select
											id={nodeFieldId('model')}
											class="mt-1 w-full rounded-md border border-gray-200 text-sm"
											value={settings?.model || 'codex'}
											onchange={(event) =>
												updateNodeSettings(selectedNodeId as string, {
													...settings,
													model: event.currentTarget.value
												})}
											disabled={!canEdit()}
										>
											<option value="codex">Codex</option>
											<option value="claude">Claude</option>
											<option value="gemini">Gemini</option>
										</select>
									</div>
									<div>
										<label
											for={nodeFieldId('system-prompt')}
											class="text-xs font-semibold uppercase text-gray-500"
										>
											System Prompt
										</label>
										<textarea
											id={nodeFieldId('system-prompt')}
											rows="4"
											class="mt-1 w-full rounded-md border border-gray-200 text-sm"
											value={settings?.systemPrompt || ''}
											oninput={(event) =>
												updateNodeSettings(selectedNodeId as string, {
													...settings,
													systemPrompt: event.currentTarget.value
												})}
											disabled={!canEdit()}
										></textarea>
									</div>
									<div class="grid grid-cols-2 gap-3">
										<div>
											<label
												for={nodeFieldId('clickup-status')}
												class="text-xs font-semibold uppercase text-gray-500"
											>
												ClickUp Status
											</label>
											<input
												id={nodeFieldId('clickup-status')}
												class="mt-1 w-full rounded-md border border-gray-200 text-sm"
												value={settings?.clickupStatus || ''}
												oninput={(event) =>
													updateNodeSettings(selectedNodeId as string, {
														...settings,
														clickupStatus: event.currentTarget.value
													})}
												disabled={!canEdit()}
											/>
										</div>
										<div>
											<label
												for={nodeFieldId('capacity')}
												class="text-xs font-semibold uppercase text-gray-500"
											>
												Capacity
											</label>
											<input
												id={nodeFieldId('capacity')}
												type="number"
												min="1"
												class="mt-1 w-full rounded-md border border-gray-200 text-sm"
												value={settings?.capacity ?? 1}
												oninput={(event) =>
													updateNodeSettings(selectedNodeId as string, {
														...settings,
														capacity: Number(event.currentTarget.value)
													})}
												disabled={!canEdit()}
											/>
										</div>
									</div>
								{:else}
									{@const settings = selectedNode()?.settings as BucketSettings}
									<div>
										<label for={nodeFieldId('bucket')} class="text-xs font-semibold uppercase text-gray-500">
											Bucket Type
										</label>
										<select
											id={nodeFieldId('bucket')}
											class="mt-1 w-full rounded-md border border-gray-200 text-sm"
											value={settings?.bucket || 'custom'}
											onchange={(event) =>
												updateNodeSettings(selectedNodeId as string, {
													...settings,
													bucket: event.currentTarget.value
												})}
											disabled={!canEdit()}
										>
											<option value="completed">Completed</option>
											<option value="failed">Failed</option>
											<option value="stopped">Stopped</option>
											<option value="custom">Custom</option>
										</select>
									</div>
								{/if}

								<button
									onclick={deleteSelectedNode}
									disabled={!canEdit()}
									class="w-full rounded-md border border-red-200 px-3 py-2 text-sm font-semibold text-red-600 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-50"
								>
									Delete Node
								</button>
							</div>
						{:else if selectedEdge()}
							<div class="mt-4 space-y-4 text-sm">
								<div>
									<label for={edgeFieldId('output')} class="text-xs font-semibold uppercase text-gray-500">
										Output
									</label>
									<select
										id={edgeFieldId('output')}
										class="mt-1 w-full rounded-md border border-gray-200 text-sm"
										value={selectedEdge()?.output || 'default'}
										onchange={(event) =>
											updateEdgeOutput(selectedEdgeId as string, event.currentTarget.value as 'default' | 'success' | 'error')}
										disabled={!canEdit()}
									>
										<option value="default">Default</option>
										<option value="success">Success (0)</option>
										<option value="error">Error (1)</option>
									</select>
								</div>
								<div>
									<div class="flex items-center justify-between">
										<span class="text-xs font-semibold uppercase text-gray-500">Actions</span>
										<button
											onclick={() => addAction(selectedEdgeId as string)}
											disabled={!canEdit()}
											class="rounded-md border border-gray-200 px-2 py-1 text-xs font-semibold text-gray-600 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
										>
											Add Action
										</button>
									</div>
									<div class="mt-3 space-y-3">
										{#each selectedEdge()?.actions || [] as action}
											<div class="rounded-md border border-gray-200 p-3">
												<div class="flex items-center justify-between">
													<select
														class="rounded-md border border-gray-200 text-sm"
														value={action.type}
														onchange={(event) =>
															updateActionType(selectedEdgeId as string, action.id, event.currentTarget.value)}
														disabled={!canEdit()}
														aria-label="Action type"
													>
														<option value="create_branch">Create Branch</option>
														<option value="update_clickup_status">Update ClickUp Status</option>
														<option value="custom">Custom</option>
													</select>
													<button
														onclick={() => removeAction(selectedEdgeId as string, action.id)}
														disabled={!canEdit()}
														class="text-xs font-semibold text-red-500 hover:text-red-600 disabled:cursor-not-allowed disabled:opacity-50"
													>
														Remove
													</button>
												</div>
												{#if action.type === 'create_branch'}
													<label
														for={actionFieldId(action.id, 'branch-prefix')}
														class="mt-2 block text-xs font-semibold uppercase text-gray-500"
													>
														Branch Prefix
													</label>
													<input
														id={actionFieldId(action.id, 'branch-prefix')}
														class="mt-1 w-full rounded-md border border-gray-200 text-sm"
														value={(action.settings?.branchPrefix as string) || ''}
														oninput={(event) =>
															updateActionSetting(selectedEdgeId as string, action.id, 'branchPrefix', event.currentTarget.value)}
														disabled={!canEdit()}
													/>
												{:else if action.type === 'update_clickup_status'}
													<label
														for={actionFieldId(action.id, 'status')}
														class="mt-2 block text-xs font-semibold uppercase text-gray-500"
													>
														Status
													</label>
													<input
														id={actionFieldId(action.id, 'status')}
														class="mt-1 w-full rounded-md border border-gray-200 text-sm"
														value={(action.settings?.status as string) || ''}
														oninput={(event) =>
															updateActionSetting(selectedEdgeId as string, action.id, 'status', event.currentTarget.value)}
														disabled={!canEdit()}
													/>
												{:else}
													<label
														for={actionFieldId(action.id, 'notes')}
														class="mt-2 block text-xs font-semibold uppercase text-gray-500"
													>
														Notes
													</label>
													<input
														id={actionFieldId(action.id, 'notes')}
														class="mt-1 w-full rounded-md border border-gray-200 text-sm"
														value={(action.settings?.notes as string) || ''}
														oninput={(event) =>
															updateActionSetting(selectedEdgeId as string, action.id, 'notes', event.currentTarget.value)}
														disabled={!canEdit()}
													/>
												{/if}
											</div>
										{/each}
									</div>
								</div>

								<button
									onclick={deleteSelectedEdge}
									disabled={!canEdit()}
									class="w-full rounded-md border border-red-200 px-3 py-2 text-sm font-semibold text-red-600 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-50"
								>
									Delete Connection
								</button>
							</div>
						{:else}
							<p class="mt-4 text-sm text-gray-500">Select a node or edge to edit its settings.</p>
						{/if}

						{#if error}
							<div class="mt-6 rounded-md bg-red-50 p-3 text-sm text-red-700">{error}</div>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</main>
</div>
