<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { saveScreenshot, generateTasks, clearScreenshots, AGENT_OPTIONS, type AgentType } from '$lib/api/voice';

	interface Screenshot {
		timestamp: number;
		filepath: string;
		filename: string;
		previewUrl: string; // Keep for UI preview
	}

	// Recording state
	let isRecording = $state(false);
	let isPaused = $state(false);
	let transcript = $state('');
	let interimTranscript = $state('');
	let error = $state<string | null>(null);

	// Screen sharing state
	let isScreenSharing = $state(false);
	let screenshots = $state<Screenshot[]>([]);
	let screenshotInterval = $state<number | null>(null);
	let isCapturing = $state(false); // Track if currently capturing a screenshot
	let isGenerating = $state(false); // Track if generating tasks

	// Agent selection
	let selectedAgent = $state<AgentType>('claude');

	// Media objects
	let recognition: SpeechRecognition | null = null;
	let screenStream: MediaStream | null = null;
	let videoElement: HTMLVideoElement | null = null;
	let canvasElement: HTMLCanvasElement | null = null;

	// Settings
	let screenshotIntervalSeconds = $state(5);

	// Check for browser support
	let speechSupported = $state(false);
	let screenShareSupported = $state(false);

	onMount(() => {
		// Check Speech Recognition support
		const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
		speechSupported = !!SpeechRecognition;

		// Check Screen Capture support
		screenShareSupported = !!navigator.mediaDevices?.getDisplayMedia;

		if (speechSupported) {
			recognition = new SpeechRecognition();
			recognition.continuous = true;
			recognition.interimResults = true;
			recognition.lang = 'en-US';

			recognition.onresult = (event) => {
				let interim = '';
				let final = '';

				for (let i = event.resultIndex; i < event.results.length; i++) {
					const result = event.results[i];
					if (result.isFinal) {
						final += result[0].transcript + ' ';
					} else {
						interim += result[0].transcript;
					}
				}

				if (final) {
					transcript += final;
				}
				interimTranscript = interim;
			};

			recognition.onerror = (event) => {
				console.error('Speech recognition error:', event.error);
				if (event.error === 'not-allowed') {
					error = 'Microphone access denied. Please allow microphone access and try again.';
				} else if (event.error === 'no-speech') {
					// Ignore no-speech errors, they're common
				} else {
					error = `Speech recognition error: ${event.error}`;
				}
			};

			recognition.onend = () => {
				// Restart if still recording (handles browser auto-stop)
				if (isRecording && !isPaused) {
					try {
						recognition?.start();
					} catch (e) {
						// Already started
					}
				}
			};
		}
	});

	onDestroy(() => {
		stopRecording();
		stopScreenShare();
	});

	async function startRecording() {
		if (!recognition) {
			error = 'Speech recognition not supported in this browser. Try Chrome or Edge.';
			return;
		}

		error = null;
		try {
			recognition.start();
			isRecording = true;
			isPaused = false;

			// Start screenshot capture if screen is being shared
			if (isScreenSharing) {
				startScreenshotCapture();
			}
		} catch (e) {
			error = 'Failed to start recording. Please check microphone permissions.';
		}
	}

	function pauseRecording() {
		if (recognition && isRecording) {
			recognition.stop();
			isPaused = true;

			// Pause screenshot capture
			stopScreenshotCapture();
		}
	}

	function resumeRecording() {
		if (recognition && isPaused) {
			try {
				recognition.start();
				isPaused = false;

				// Resume screenshot capture if screen is being shared
				if (isScreenSharing) {
					startScreenshotCapture();
				}
			} catch (e) {
				error = 'Failed to resume recording.';
			}
		}
	}

	function stopRecording() {
		if (recognition) {
			recognition.stop();
		}
		isRecording = false;
		isPaused = false;
		interimTranscript = '';

		// Stop screenshot capture
		stopScreenshotCapture();
	}

	async function startScreenShare() {
		if (!screenShareSupported) {
			error = 'Screen sharing not supported in this browser.';
			return;
		}

		error = null;
		try {
			screenStream = await navigator.mediaDevices.getDisplayMedia({
				video: true,
				audio: false
			});

			isScreenSharing = true;

			// Set up video element
			if (videoElement) {
				videoElement.srcObject = screenStream;
				videoElement.play();
			}

			// Listen for when user stops sharing via browser UI
			screenStream.getVideoTracks()[0].onended = () => {
				stopScreenShare();
			};

			// Only start taking screenshots if we're actively recording (not paused)
			if (isRecording && !isPaused) {
				startScreenshotCapture();
			}
		} catch (e) {
			if ((e as Error).name === 'NotAllowedError') {
				error = 'Screen sharing was cancelled or denied.';
			} else {
				error = 'Failed to start screen sharing.';
			}
		}
	}

	function stopScreenshotCapture() {
		if (screenshotInterval) {
			clearInterval(screenshotInterval);
			screenshotInterval = null;
		}
	}

	function stopScreenShare() {
		stopScreenshotCapture();

		if (screenStream) {
			screenStream.getTracks().forEach(track => track.stop());
			screenStream = null;
		}

		if (videoElement) {
			videoElement.srcObject = null;
		}

		isScreenSharing = false;
	}

	function startScreenshotCapture() {
		// Clear any existing interval first
		stopScreenshotCapture();

		// Take initial screenshot
		captureScreenshot();

		// Set up interval
		screenshotInterval = window.setInterval(() => {
			captureScreenshot();
		}, screenshotIntervalSeconds * 1000);
	}

	async function captureScreenshot() {
		if (!videoElement || !canvasElement || !isScreenSharing || isCapturing) return;

		const ctx = canvasElement.getContext('2d');
		if (!ctx) return;

		isCapturing = true;

		try {
			// Set canvas size to match video
			canvasElement.width = videoElement.videoWidth;
			canvasElement.height = videoElement.videoHeight;

			// Draw current frame
			ctx.drawImage(videoElement, 0, 0);

			// Get data URL for preview and upload
			const dataUrl = canvasElement.toDataURL('image/jpeg', 0.8);
			const timestamp = Date.now();

			// Upload to backend
			const response = await saveScreenshot(dataUrl, `screenshot_${timestamp}.jpg`);

			// Add to screenshots with filepath from backend
			screenshots = [...screenshots, {
				timestamp,
				filepath: response.filepath,
				filename: response.filename,
				previewUrl: dataUrl
			}];
		} catch (e) {
			console.error('Failed to save screenshot:', e);
			// Don't show error to user for individual screenshot failures
		} finally {
			isCapturing = false;
		}
	}

	async function clearSession() {
		stopRecording();
		stopScreenShare();
		transcript = '';
		interimTranscript = '';
		screenshots = [];
		error = null;

		// Clear screenshots from backend
		try {
			await clearScreenshots();
		} catch (e) {
			console.error('Failed to clear screenshots from server:', e);
		}
	}

	async function handleGenerateTasks() {
		if (!transcript.trim()) {
			error = 'Please record some audio first.';
			return;
		}

		isGenerating = true;
		error = null;

		try {
			const screenshotPaths = screenshots.map(s => s.filepath);
			const response = await generateTasks(transcript, screenshotPaths, selectedAgent);

			if (response.success) {
				// Show success message
				error = null;
				// Could navigate to a results page or show a success modal
				const agentLabel = AGENT_OPTIONS.find(a => a.value === selectedAgent)?.label || selectedAgent;
				alert(`${agentLabel} agent started! ${response.message}`);
			} else {
				error = response.message || 'Failed to generate tasks';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to generate tasks';
		} finally {
			isGenerating = false;
		}
	}

	function removeScreenshot(index: number) {
		screenshots = screenshots.filter((_, i) => i !== index);
	}

	function formatTime(timestamp: number): string {
		return new Date(timestamp).toLocaleTimeString();
	}

	// Computed state for display
	let fullTranscript = $derived(transcript + interimTranscript);
	let hasContent = $derived(transcript.length > 0 || screenshots.length > 0);
</script>

<!-- Hidden elements for screen capture -->
<video bind:this={videoElement} class="hidden" muted playsinline></video>
<canvas bind:this={canvasElement} class="hidden"></canvas>

<div class="space-y-6">
	<!-- Error Banner -->
	{#if error}
		<div class="rounded-md bg-red-50 p-4">
			<div class="flex">
				<div class="flex-shrink-0">
					<svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
					</svg>
				</div>
				<div class="ml-3">
					<p class="text-sm text-red-700">{error}</p>
				</div>
				<div class="ml-auto pl-3">
					<button onclick={() => error = null} class="text-red-500 hover:text-red-600">
						<svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
						</svg>
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Controls -->
	<div class="rounded-lg bg-white p-6 shadow">
		<div class="flex flex-wrap items-center gap-4">
			<!-- Microphone Controls -->
			<div class="flex items-center gap-2">
				{#if !isRecording}
					<button
						onclick={startRecording}
						disabled={!speechSupported}
						class="inline-flex items-center gap-2 rounded-full bg-red-600 px-6 py-3 text-sm font-semibold text-white shadow-sm hover:bg-red-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
							<path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
							<path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
						</svg>
						Start Recording
					</button>
				{:else}
					{#if isPaused}
						<button
							onclick={resumeRecording}
							class="inline-flex items-center gap-2 rounded-full bg-green-600 px-4 py-3 text-sm font-semibold text-white shadow-sm hover:bg-green-500 transition-colors"
						>
							<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
								<path d="M8 5v14l11-7z"/>
							</svg>
							Resume
						</button>
					{:else}
						<button
							onclick={pauseRecording}
							class="inline-flex items-center gap-2 rounded-full bg-yellow-600 px-4 py-3 text-sm font-semibold text-white shadow-sm hover:bg-yellow-500 transition-colors"
						>
							<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
								<path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
							</svg>
							Pause
						</button>
					{/if}
					<button
						onclick={stopRecording}
						class="inline-flex items-center gap-2 rounded-full bg-gray-600 px-4 py-3 text-sm font-semibold text-white shadow-sm hover:bg-gray-500 transition-colors"
					>
						<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
							<path d="M6 6h12v12H6z"/>
						</svg>
						Stop
					</button>
				{/if}

				{#if isRecording && !isPaused}
					<span class="flex items-center gap-2 text-sm text-red-600">
						<span class="relative flex h-3 w-3">
							<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"></span>
							<span class="relative inline-flex rounded-full h-3 w-3 bg-red-500"></span>
						</span>
						Recording...
					</span>
				{/if}
			</div>

			<div class="h-8 w-px bg-gray-300"></div>

			<!-- Screen Share Controls -->
			<div class="flex items-center gap-2">
				{#if !isScreenSharing}
					<button
						onclick={startScreenShare}
						disabled={!screenShareSupported}
						class="inline-flex items-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
						</svg>
						Share Screen
					</button>
				{:else}
					<button
						onclick={stopScreenShare}
						class="inline-flex items-center gap-2 rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-red-500 transition-colors"
					>
						<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						</svg>
						Stop Sharing
					</button>
					{#if isRecording && !isPaused}
						<span class="flex items-center gap-2 text-sm text-indigo-600">
							<span class="relative flex h-3 w-3">
								<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75"></span>
								<span class="relative inline-flex rounded-full h-3 w-3 bg-indigo-500"></span>
							</span>
							Capturing every {screenshotIntervalSeconds}s
						</span>
					{:else}
						<span class="flex items-center gap-2 text-sm text-gray-500">
							<span class="relative flex h-3 w-3">
								<span class="relative inline-flex rounded-full h-3 w-3 bg-gray-400"></span>
							</span>
							{#if !isRecording}
								Start recording to capture
							{:else}
								Paused
							{/if}
						</span>
					{/if}
				{/if}
			</div>

			<div class="ml-auto">
				{#if hasContent}
					<button
						onclick={clearSession}
						class="inline-flex items-center gap-2 rounded-lg bg-gray-100 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-200 transition-colors"
					>
						<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
						</svg>
						Clear All
					</button>
				{/if}
			</div>
		</div>

		<!-- Screenshot interval setting -->
		{#if isScreenSharing}
			<div class="mt-4 pt-4 border-t border-gray-200">
				<label class="flex items-center gap-3 text-sm text-gray-600">
					Screenshot interval:
					<select
						bind:value={screenshotIntervalSeconds}
						class="rounded-md border-gray-300 text-sm shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
					>
						<option value={3}>3 seconds</option>
						<option value={5}>5 seconds</option>
						<option value={10}>10 seconds</option>
						<option value={15}>15 seconds</option>
					</select>
				</label>
			</div>
		{/if}
	</div>

	<!-- Transcription -->
	<div class="rounded-lg bg-white p-6 shadow">
		<h2 class="text-lg font-medium text-gray-900 mb-4">Transcription</h2>
		<div class="min-h-[200px] max-h-[400px] overflow-y-auto rounded-lg bg-gray-50 p-4">
			{#if fullTranscript}
				<p class="text-gray-800 whitespace-pre-wrap leading-relaxed">
					{transcript}<span class="text-gray-400">{interimTranscript}</span>
				</p>
			{:else}
				<p class="text-gray-400 italic">
					{#if isRecording}
						Listening... Start speaking to see your transcription here.
					{:else}
						Click "Start Recording" to begin transcribing your voice.
					{/if}
				</p>
			{/if}
		</div>
	</div>

	<!-- Screenshots -->
	{#if screenshots.length > 0}
		<div class="rounded-lg bg-white p-6 shadow">
			<h2 class="text-lg font-medium text-gray-900 mb-4">
				Screenshots ({screenshots.length})
			</h2>
			<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
				{#each screenshots as screenshot, index}
					<div class="group relative rounded-lg overflow-hidden border border-gray-200 bg-gray-50">
						<img
							src={screenshot.previewUrl}
							alt="Screenshot at {formatTime(screenshot.timestamp)}"
							class="w-full h-32 object-cover"
						/>
						<div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
							<button
								onclick={() => window.open(screenshot.previewUrl, '_blank')}
								class="p-2 bg-white/90 rounded-full hover:bg-white transition-colors"
								title="View full size"
							>
								<svg class="h-4 w-4 text-gray-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" />
								</svg>
							</button>
							<button
								onclick={() => removeScreenshot(index)}
								class="p-2 bg-white/90 rounded-full hover:bg-white transition-colors"
								title="Remove"
							>
								<svg class="h-4 w-4 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
								</svg>
							</button>
						</div>
						<div class="p-2 text-xs text-gray-500">
							<div>{formatTime(screenshot.timestamp)}</div>
							<div class="truncate text-gray-400" title={screenshot.filepath}>@{screenshot.filepath}</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Generate Tasks Button -->
	{#if hasContent}
		<div class="rounded-lg bg-gradient-to-r from-indigo-500 to-purple-600 p-6 shadow">
			<div class="flex flex-col gap-4">
				<div class="flex items-center justify-between">
					<div>
						<h3 class="text-lg font-medium text-white">Ready to generate tasks?</h3>
						<p class="text-sm text-indigo-100 mt-1">
							The agent will analyze your recording
							{#if screenshots.length > 0}
								and {screenshots.length} screenshot{screenshots.length > 1 ? 's' : ''}
							{/if}
							to create actionable tasks.
						</p>
					</div>
					<button
						onclick={handleGenerateTasks}
						disabled={isGenerating || !transcript.trim()}
						class="inline-flex items-center gap-2 rounded-lg bg-white px-6 py-3 text-sm font-semibold text-indigo-600 shadow-sm hover:bg-indigo-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						{#if isGenerating}
							<svg class="h-5 w-5 animate-spin" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							Generating...
						{:else}
							<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
							</svg>
							Generate Tasks
						{/if}
					</button>
				</div>

				<!-- Agent Selection -->
				<div class="flex items-center gap-3 pt-3 border-t border-white/20">
					<label class="text-sm font-medium text-white/90">Agent:</label>
					<div class="flex gap-2">
						{#each AGENT_OPTIONS as agent}
							<button
								onclick={() => selectedAgent = agent.value}
								class="px-4 py-2 rounded-lg text-sm font-medium transition-all {selectedAgent === agent.value
									? 'bg-white text-indigo-600 shadow-md'
									: 'bg-white/20 text-white hover:bg-white/30'}"
								title={agent.description}
							>
								{agent.label}
							</button>
						{/each}
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Browser Support Info -->
	{#if !speechSupported || !screenShareSupported}
		<div class="rounded-md bg-amber-50 p-4">
			<div class="flex">
				<div class="flex-shrink-0">
					<svg class="h-5 w-5 text-amber-400" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
					</svg>
				</div>
				<div class="ml-3">
					<h3 class="text-sm font-medium text-amber-800">Limited Browser Support</h3>
					<div class="mt-2 text-sm text-amber-700">
						<ul class="list-disc pl-5 space-y-1">
							{#if !speechSupported}
								<li>Speech recognition requires Chrome, Edge, or Safari</li>
							{/if}
							{#if !screenShareSupported}
								<li>Screen sharing requires a modern browser with getDisplayMedia support</li>
							{/if}
						</ul>
					</div>
				</div>
			</div>
		</div>
	{/if}
</div>
