<script lang="ts">
	interface Props {
		isOpen?: boolean;
		onClose?: () => void;
	}

	let { isOpen = false, onClose }: Props = $props();
</script>

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50" role="dialog">
		<div class="bg-white rounded-lg shadow-lg max-w-md w-full mx-4">
			<!-- Header -->
			<div class="flex items-center justify-between p-6 border-b border-gray-200">
				<h2 class="text-lg font-semibold text-gray-900">Quick Start Project</h2>
				<button
					onclick={onClose}
					class="text-gray-400 hover:text-gray-600 transition-colors"
					aria-label="Close"
				>
					<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<!-- Body -->
			<div class="p-6 space-y-4">
				<p class="text-sm text-gray-600">Create a new project to get started managing your workflows.</p>

				<div class="space-y-3">
					<a
						href="/projects/new"
						class="flex items-center gap-3 p-3 rounded-lg border border-gray-200 hover:border-indigo-300 hover:bg-indigo-50 transition-colors"
						onclick={onClose}
					>
						<div class="flex-shrink-0">
							<svg class="h-5 w-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M12 4v16m8-8H4"
								/>
							</svg>
						</div>
						<div>
							<div class="font-medium text-gray-900">Create New Project</div>
							<div class="text-xs text-gray-500">Start fresh with a new repository</div>
						</div>
					</a>

					<a
						href="/projects/new?mode=import"
						class="flex items-center gap-3 p-3 rounded-lg border border-gray-200 hover:border-indigo-300 hover:bg-indigo-50 transition-colors"
						onclick={onClose}
					>
						<div class="flex-shrink-0">
							<svg class="h-5 w-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
								/>
							</svg>
						</div>
						<div>
							<div class="font-medium text-gray-900">Import from GitHub</div>
							<div class="text-xs text-gray-500">Clone a repository to get started</div>
						</div>
					</a>
				</div>
			</div>

			<!-- Footer -->
			<div class="flex justify-end gap-2 p-6 border-t border-gray-200">
				<button
					onclick={onClose}
					class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
				>
					Cancel
				</button>
			</div>
		</div>
	</div>
{/if}
