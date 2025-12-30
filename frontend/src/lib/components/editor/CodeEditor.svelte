<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	interface Props {
		content: string;
		language: string;
		onContentChange?: (content: string) => void;
	}

	let { content, language, onContentChange }: Props = $props();

	let container: HTMLDivElement;
	let editor: any = null;
	let monaco: any = null;

	onMount(async () => {
		// Dynamically import Monaco
		try {
			monaco = await import('monaco-editor');

			// Configure Monaco environment for web workers
			self.MonacoEnvironment = {
				getWorker: function (_moduleId: string, label: string) {
					if (label === 'json') {
						return new Worker(new URL('monaco-editor/esm/vs/language/json/json.worker.js', import.meta.url), { type: 'module' });
					}
					if (label === 'css' || label === 'scss' || label === 'less') {
						return new Worker(new URL('monaco-editor/esm/vs/language/css/css.worker.js', import.meta.url), { type: 'module' });
					}
					if (label === 'html' || label === 'handlebars' || label === 'razor') {
						return new Worker(new URL('monaco-editor/esm/vs/language/html/html.worker.js', import.meta.url), { type: 'module' });
					}
					if (label === 'typescript' || label === 'javascript') {
						return new Worker(new URL('monaco-editor/esm/vs/language/typescript/ts.worker.js', import.meta.url), { type: 'module' });
					}
					return new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url), { type: 'module' });
				}
			};

			editor = monaco.editor.create(container, {
				value: content,
				language: language,
				theme: 'vs-dark',
				automaticLayout: true,
				minimap: { enabled: true },
				fontSize: 13,
				fontFamily: 'Menlo, Monaco, "Courier New", monospace',
				scrollBeyondLastLine: false,
				renderWhitespace: 'selection',
				tabSize: 2,
				wordWrap: 'on',
				lineNumbers: 'on',
				folding: true,
				bracketPairColorization: { enabled: true },
				padding: { top: 10 }
			});

			editor.onDidChangeModelContent(() => {
				onContentChange?.(editor.getValue());
			});
		} catch (e) {
			console.error('Failed to load Monaco Editor:', e);
		}
	});

	onDestroy(() => {
		editor?.dispose();
	});

	// Update content when prop changes (for switching between tabs)
	$effect(() => {
		if (editor && editor.getValue() !== content) {
			const model = editor.getModel();
			if (model) {
				model.setValue(content);
			}
		}
	});

	// Update language when prop changes
	$effect(() => {
		if (editor && monaco) {
			const model = editor.getModel();
			if (model) {
				monaco.editor.setModelLanguage(model, language);
			}
		}
	});
</script>

<div bind:this={container} class="h-full w-full"></div>

<style>
	div {
		/* Ensure Monaco takes full height */
		min-height: 100%;
	}
</style>
