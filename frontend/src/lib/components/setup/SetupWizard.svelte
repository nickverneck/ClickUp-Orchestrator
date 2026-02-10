<script lang="ts">
	import { goto } from '$app/navigation';
	import type { SetupStatus } from '$lib/api/setup';

	interface Props {
		status: SetupStatus;
		onComplete: () => void;
	}

	let { status, onComplete }: Props = $props();
</script>

<div class="min-h-screen bg-gradient-to-br from-indigo-50 to-white">
	<div class="mx-auto flex max-w-2xl flex-col items-center justify-center px-4 py-20 sm:px-6 lg:px-8">
		<!-- Logo/Icon -->
		<div class="mb-8 flex h-16 w-16 items-center justify-center rounded-full bg-indigo-100">
			<svg class="h-8 w-8 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M13 10V3L4 14h7v7l9-11h-7z"
				/>
			</svg>
		</div>

		<!-- Heading -->
		<h1 class="text-center text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
			Welcome to ClickUp Orchestrator
		</h1>

		<p class="mt-4 text-center text-lg text-gray-600">
			Automate your workflow by connecting ClickUp tasks with Claude AI agents
		</p>

		<!-- Features -->
		<div class="mt-12 grid w-full max-w-lg gap-6 sm:grid-cols-3">
			<div class="rounded-lg bg-white p-4 text-center shadow-sm">
				<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-indigo-100 mx-auto">
					<svg class="h-5 w-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
					</svg>
				</div>
				<p class="text-sm font-medium text-gray-900">Multi-Project Support</p>
				<p class="mt-1 text-xs text-gray-500">Manage multiple ClickUp workspaces</p>
			</div>

			<div class="rounded-lg bg-white p-4 text-center shadow-sm">
				<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-indigo-100 mx-auto">
					<svg class="h-5 w-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
					</svg>
				</div>
				<p class="text-sm font-medium text-gray-900">AI Powered</p>
				<p class="mt-1 text-xs text-gray-500">Powered by Claude AI</p>
			</div>

			<div class="rounded-lg bg-white p-4 text-center shadow-sm">
				<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-indigo-100 mx-auto">
					<svg class="h-5 w-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4m0 0l1 1m-1-1l-1 1m1-1l1-1m-1 1l-1-1" />
					</svg>
				</div>
				<p class="text-sm font-medium text-gray-900">Git Integration</p>
				<p class="mt-1 text-xs text-gray-500">Automatic worktree management</p>
			</div>
		</div>

		<!-- CTA -->
		<div class="mt-12 w-full max-w-lg space-y-4">
			<button
				onclick={() => goto('/projects/new')}
				class="w-full rounded-lg bg-indigo-600 px-6 py-3 text-center font-semibold text-white hover:bg-indigo-500"
			>
				Create Your First Project
			</button>

			<p class="text-center text-sm text-gray-500">
				Each project can have its own ClickUp workspace and AI configuration
			</p>
		</div>

		<!-- Info Box -->
		<div class="mt-10 w-full max-w-lg rounded-lg bg-blue-50 p-4">
			<div class="flex gap-3">
				<div class="flex-shrink-0">
					<svg class="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
						<path
							fill-rule="evenodd"
							d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
							clip-rule="evenodd"
						/>
					</svg>
				</div>
				<div class="text-sm text-blue-700">
					<p class="font-medium">Getting Started</p>
					<p class="mt-1">To set up a project, you'll need a ClickUp API key. You can create one in your ClickUp account settings.</p>
				</div>
			</div>
		</div>
	</div>
</div>
