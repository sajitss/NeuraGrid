<script>
  import { onMount } from 'svelte';

  let socket;
  let connected = $state(false);
  let workers = $state([]);
  let jobs = $state([]);
  let stats = $state({
    activeWorkers: 0,
    totalTflops: 0,
    jobsCompleted: 0
  });
  let showQueue = $state(false);
  let queueData = $state({});

  onMount(() => {
    connectWebSocket();
    fetchQueue(); // Fetch initial queue data
    // Poll queue every 5 seconds
    setInterval(fetchQueue, 5000);
  });

  function connectWebSocket() {
    socket = new WebSocket('ws://localhost:3000/ws');

    socket.onopen = () => {
      connected = true;
      console.log('Connected to Coordinator');
    };

    socket.onclose = () => {
      connected = false;
      console.log('Disconnected');
      setTimeout(connectWebSocket, 5000);
    };

    socket.onmessage = (event) => {
      const data = JSON.parse(event.data);
      handleMessage(data);
    };
  }

  async function fetchQueue() {
    try {
        const res = await fetch('http://localhost:3000/api/queue');
        queueData = await res.json();
    } catch (e) {
        console.error("Failed to fetch queue", e);
    }
  }

  function toggleQueue() {
    showQueue = !showQueue;
    if (showQueue) {
        fetchQueue();
    }
  }

  function handleMessage(data) {
    if (data.type === 'stats') {
      stats = data.payload;
    } else if (data.type === 'workers') {
      workers = data.payload;
    } else if (data.type === 'job_update') {
      jobs = [data.payload, ...jobs].slice(0, 50);
    } else if (data.job_type) {
      // Handle raw job broadcast
      const newJob = {
        id: data.id || 'new-job',
        timestamp: new Date(),
        message: `New Job: ${data.job_type} (${JSON.stringify(data.args)})`,
        status: 'pending'
      };
      jobs = [newJob, ...jobs].slice(0, 50);
    }
  }

  // Mock data for UI development or Demo Mode
  if (!connected || workers.length === 0) {
    workers = [
      { id: 'w1', hostname: 'Worker-Titan', role: 'LLM Training', gpu: 'NVIDIA H100 (80GB)', cpu: 'AMD EPYC 9654', status: 'busy', task: 'Llama-3 Fine-tuning' },
      { id: 'w2', hostname: 'Worker-Alpha', role: 'Inference', gpu: 'RTX 4090 (24GB)', cpu: 'i9-14900K', status: 'busy', task: 'Stable Diffusion XL' },
      { id: 'w3', hostname: 'Worker-Beta', role: '3D Rendering', gpu: 'RTX A6000 (48GB)', cpu: 'Threadripper 7980X', status: 'idle', task: '-' },
      { id: 'w4', hostname: 'Worker-Gamma', role: 'Small Inference', gpu: 'RTX 3060 (12GB)', cpu: 'Ryzen 5 5600X', status: 'busy', task: 'String Search' },
      { id: 'w5', hostname: 'Worker-Delta', role: 'Mobile Dev', gpu: 'RTX 4070 Mobile', cpu: 'i7-13700H', status: 'idle', task: '-' },
      { id: 'w6', hostname: 'Worker-Epsilon', role: 'Local LLM', gpu: 'Apple M2 Ultra', cpu: 'ARM64', status: 'busy', task: 'Mistral 7B' },
      { id: 'w7', hostname: 'Worker-Zeta', role: 'Edge AI', gpu: 'Adreno 750', cpu: 'Snapdragon 8 Gen 3', status: 'idle', task: '-' },
      { id: 'w8', hostname: 'Worker-Eta', role: 'IoT Gateway', gpu: 'Ampere (2048 Cores)', cpu: 'Orin AGX', status: 'busy', task: 'Sensor Fusion' },
      { id: 'w9', hostname: 'Worker-Kappa', role: 'Education', gpu: 'Maxwell (128 Cores)', cpu: 'Jetson Nano', status: 'offline', task: '-' },
      { id: 'w10', hostname: 'Worker-Lambda', role: 'Industrial', gpu: 'Volta (384 Cores)', cpu: 'Xavier NX', status: 'idle', task: '-' },
    ];
    stats = { activeWorkers: 10, totalTflops: 2450.5, jobsCompleted: 14203 };
  }

  let capabilities = [
    { code: 'Pd', title: 'People Detection', desc: 'timestamps of person appearances, or True/False if any person present', style: 'border-cyan-500 text-cyan-400 hover:bg-cyan-500/10 hover:shadow-[0_0_10px_rgba(6,182,212,0.5)]', count: 8 },
    { code: 'Pr', title: 'Person Re‑Identification', desc: 'timestamps where a specific person reappears across cameras', style: 'border-purple-500 text-purple-400 hover:bg-purple-500/10 hover:shadow-[0_0_10px_rgba(168,85,247,0.5)]', count: 3 },
    { code: 'Ot', title: 'Object Tracking', desc: 'timestamps of object movement across frames', style: 'border-emerald-500 text-emerald-400 hover:bg-emerald-500/10 hover:shadow-[0_0_10px_rgba(16,185,129,0.5)]', count: 6 },
    { code: 'Vd', title: 'Vehicle Detection', desc: 'timestamps of car appearances, or True/False if any car present', style: 'border-orange-500 text-orange-400 hover:bg-orange-500/10 hover:shadow-[0_0_10px_rgba(249,115,22,0.5)]', count: 4 },
    { code: 'Lp', title: 'License Plate Recognition', desc: 'timestamps of a specific plate, or True/False if matched', style: 'border-rose-500 text-rose-400 hover:bg-rose-500/10 hover:shadow-[0_0_10px_rgba(244,63,94,0.5)]', count: 2 },
    { code: 'Vs', title: 'Video Search / Retrieval', desc: 'timestamps of query match, or True/False if found', style: 'border-fuchsia-500 text-fuchsia-400 hover:bg-fuchsia-500/10 hover:shadow-[0_0_10px_rgba(217,70,239,0.5)]', count: 5 }
  ];
</script>

<main class="min-h-screen bg-hpc-dark text-gray-200 p-8 font-sans">
  <!-- Header -->
  <header class="flex justify-between items-center mb-8 border-b border-gray-800 pb-4">
    <div class="flex items-center gap-4">
      <img src="/logo.png" alt="NeuraGrid Logo" class="h-12 w-auto object-contain" />
      <h1 class="text-3xl font-bold tracking-wider text-transparent bg-clip-text bg-gradient-to-r from-hpc-cyan to-purple-500">
        NeuraGrid Dashboard <span class="text-sm font-mono text-gray-500">v1.0</span>
      </h1>
      <div class="w-3 h-3 rounded-full {connected ? 'bg-hpc-green shadow-[0_0_10px_#00ff9d]' : 'bg-red-500'}"></div>
    </div>
    <div class="font-mono text-sm text-hpc-cyan">
      {new Date().toLocaleTimeString()}
    </div>
  </header>

  <!-- Stats Grid -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
    <div class="bg-hpc-blue/50 backdrop-blur border border-gray-700 p-6 rounded-xl relative overflow-hidden group">
      <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
        <svg class="w-16 h-16" fill="currentColor" viewBox="0 0 20 20"><path d="M13 6a3 3 0 11-6 0 3 3 0 016 0zM18 8a2 2 0 11-4 0 2 2 0 014 0zM14 15a4 4 0 00-8 0v3h8v-3zM6 8a2 2 0 11-4 0 2 2 0 014 0zM16 18v-3a5.972 5.972 0 00-.75-2.906A3.005 3.005 0 0119 15v3h-3zM4.75 12.094A5.973 5.973 0 004 15v3H1v-3a3 3 0 013.75-2.906z"></path></svg>
      </div>
      <h3 class="text-gray-400 text-sm font-mono uppercase">Active Workers</h3>
      <p class="text-4xl font-bold text-white mt-2">{stats.activeWorkers}</p>
    </div>

    <div class="bg-hpc-blue/50 backdrop-blur border border-gray-700 p-6 rounded-xl relative overflow-hidden group">
      <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
        <svg class="w-16 h-16" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clip-rule="evenodd"></path></svg>
      </div>
      <h3 class="text-gray-400 text-sm font-mono uppercase">Total Compute (TFLOPS)</h3>
      <p class="text-4xl font-bold text-hpc-cyan mt-2">{stats.totalTflops}</p>
    </div>

    <div class="bg-hpc-blue/50 backdrop-blur border border-gray-700 p-6 rounded-xl relative overflow-hidden group">
      <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
        <svg class="w-16 h-16" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path></svg>
      </div>
      <h3 class="text-gray-400 text-sm font-mono uppercase">Jobs Completed</h3>
      <p class="text-4xl font-bold text-hpc-green mt-2">{stats.jobsCompleted}</p>
    </div>
  </div>

  <!-- Main Content Area -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
    
    <!-- Worker Grid -->
    <div class="lg:col-span-2">
      <h2 class="text-xl font-bold mb-4 flex items-center gap-2">
        <span class="text-hpc-cyan">///</span> WORKER NODES
      </h2>
      <!-- Compact Grid: 2 columns on medium, 3 on large, 4 on xl -->
      <div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-3">
        {#each workers as worker}
          <div class="group bg-hpc-blue border border-gray-700 rounded-lg p-3 hover:border-hpc-cyan transition-all relative overflow-hidden cursor-default h-24 hover:h-auto hover:z-10 hover:shadow-2xl hover:bg-gray-800">
            
            <!-- Compact View -->
            <div class="flex justify-between items-start mb-2">
              <div class="truncate">
                <h3 class="font-bold text-sm text-gray-200 truncate">{worker.hostname}</h3>
                <p class="text-[10px] text-gray-500 font-mono uppercase">{worker.role || 'Worker Node'}</p>
              </div>
              <div class="w-2 h-2 rounded-full {worker.status === 'busy' ? 'bg-yellow-500 animate-pulse' : worker.status === 'offline' ? 'bg-red-500' : 'bg-green-500'}"></div>
            </div>
            
            <!-- Progress Bar (Always Visible) -->
            <div class="h-1 bg-gray-700 rounded-full overflow-hidden mb-2">
              <div class="h-full bg-hpc-cyan" style="width: {worker.status === 'busy' ? '80%' : '5%'}"></div>
            </div>

            <!-- Expanded Details (Visible on Hover) -->
            <div class="hidden group-hover:block space-y-1 pt-2 border-t border-gray-700 mt-2">
              <div class="flex justify-between text-[10px]">
                <span class="text-gray-500">GPU</span>
                <span class="font-mono text-hpc-cyan">{worker.gpu}</span>
              </div>
              <div class="flex justify-between text-[10px]">
                <span class="text-gray-500">CPU</span>
                <span class="font-mono text-gray-300">{worker.cpu}</span>
              </div>
              <div class="flex justify-between text-[10px]">
                <span class="text-gray-500">Task</span>
                <span class="font-mono text-yellow-500 truncate">{worker.task}</span>
              </div>
            </div>

          </div>
        {/each}
      </div>

      <!-- System Log -->
      <div class="mt-8 bg-hpc-blue/30 border border-gray-800 rounded-xl p-6 h-[500px] flex flex-col">
        <h2 class="text-xl font-bold mb-4 flex items-center gap-2">
          <span class="text-purple-500">///</span> SYSTEM LOG
        </h2>
        <div class="flex-1 overflow-y-auto font-mono text-sm space-y-2 pr-2 custom-scrollbar">
          {#each jobs as job}
            <div class="p-2 border-l-2 {job.status === 'failed' ? 'border-red-500 bg-red-500/10' : 'border-hpc-green bg-green-500/5'}">
              <div class="flex justify-between text-xs text-gray-500 mb-1">
                <span>{job.id.slice(0,8)}</span>
                <span>{new Date(job.timestamp).toLocaleTimeString()}</span>
              </div>
              <p class="text-gray-300">{job.message}</p>
            </div>
          {/each}
          {#if jobs.length === 0}
            <div class="text-gray-600 text-center mt-10">No recent activity</div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Right Column -->
    <div class="space-y-6">
      
      <!-- Capabilities Card -->
      <div class="bg-hpc-blue/30 border border-gray-800 rounded-xl p-6">
        <h2 class="text-xl font-bold mb-4 flex items-center gap-2">
          <span class="text-hpc-green">///</span> GRID CAPABILITIES
        </h2>
        <div class="flex flex-wrap gap-2">
          {#each capabilities as cap}
            <div class="group relative">
              <div class="w-10 h-10 flex items-center justify-center bg-gray-800/50 rounded-lg text-sm font-bold cursor-help transition-all duration-300 border {cap.style} relative">
                {cap.code}
                <div class="absolute -top-2 -right-2 bg-gray-900 border border-gray-600 text-[9px] text-gray-300 rounded-full w-4 h-4 flex items-center justify-center shadow-sm">
                  {cap.count}
                </div>
              </div>
              <!-- Tooltip -->
              <div class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 w-48 p-2 bg-gray-900 border border-gray-600 rounded shadow-xl text-xs text-gray-300 z-10 hidden group-hover:block pointer-events-none">
                <strong class="block text-blue-400 mb-1">{cap.title}</strong>
                {cap.desc}
                <div class="absolute top-full left-1/2 transform -translate-x-1/2 -mt-1 border-4 border-transparent border-t-gray-600"></div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Pending Tags Queue -->
      <div class="bg-hpc-blue/30 border border-gray-800 rounded-xl p-6">
        <h4 class="text-xl font-bold mb-4 flex justify-between items-center">
            <span class="flex items-center gap-2"><span class="text-blue-500">///</span> PENDING TAGS</span>
            <button onclick={fetchQueue} class="text-xs text-hpc-cyan hover:underline font-mono">REFRESH</button>
        </h4>
        <div class="flex-1 overflow-y-auto custom-scrollbar space-y-3 max-h-[300px]">
            {#each Object.entries(queueData) as [tag, count]}
                <div>
                    <div class="flex justify-between text-xs mb-1">
                        <span class="font-mono text-blue-400">{tag}</span>
                        <span class="text-gray-400">{count}</span>
                    </div>
                    <!-- Queue Depth Line -->
                    <div class="h-1 bg-gray-800 rounded-full overflow-hidden">
                        <div class="h-full bg-blue-500" style="width: {Math.min(count * 5, 100)}%"></div>
                    </div>
                </div>
            {/each}
            {#if Object.keys(queueData).length === 0}
                <div class="text-xs text-gray-500 text-center py-4">No pending tags</div>
            {/if}
        </div>
      </div>

  </div>
</main>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: #0a0f1c;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #374151;
    border-radius: 3px;
  }
</style>
