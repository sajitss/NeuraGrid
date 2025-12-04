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

  onMount(() => {
    connectWebSocket();
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

  // Mock data for UI development if no backend connection
  if (!connected) {
    workers = [
      { id: 'w1', hostname: 'Worker-Alpha', ip: '192.168.1.101', gpu: 'RTX 4090', status: 'busy', task: 'Prime Search' },
      { id: 'w2', hostname: 'Worker-Beta', ip: '192.168.1.102', gpu: 'RTX 3080', status: 'idle', task: '-' },
      { id: 'w3', hostname: 'Worker-Gamma', ip: '192.168.1.103', gpu: 'A100', status: 'busy', task: 'String Search' },
    ];
    stats = { activeWorkers: 3, totalTflops: 125.5, jobsCompleted: 1420 };
  }
</script>

<main class="min-h-screen bg-hpc-dark text-gray-200 p-8 font-sans">
  <!-- Header -->
  <header class="flex justify-between items-center mb-8 border-b border-gray-800 pb-4">
    <div class="flex items-center gap-4">
      <div class="w-3 h-3 rounded-full {connected ? 'bg-hpc-green shadow-[0_0_10px_#00ff9d]' : 'bg-red-500'}"></div>
      <h1 class="text-3xl font-bold tracking-wider text-transparent bg-clip-text bg-gradient-to-r from-hpc-cyan to-purple-500">
        NEURAGRID <span class="text-sm font-mono text-gray-500">COORDINATOR v1.0</span>
      </h1>
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
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#each workers as worker}
          <div class="bg-hpc-blue border border-gray-700 rounded-lg p-4 hover:border-hpc-cyan transition-colors relative overflow-hidden">
            <div class="flex justify-between items-start mb-4">
              <div>
                <h3 class="font-bold text-lg">{worker.hostname}</h3>
                <p class="text-xs text-gray-500 font-mono">{worker.ip}</p>
              </div>
              <span class="px-2 py-1 rounded text-xs font-bold uppercase {worker.status === 'busy' ? 'bg-yellow-500/20 text-yellow-500' : 'bg-green-500/20 text-green-500'}">
                {worker.status}
              </span>
            </div>
            
            <div class="space-y-2">
              <div class="flex justify-between text-sm">
                <span class="text-gray-500">GPU</span>
                <span class="font-mono text-hpc-cyan">{worker.gpu}</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-gray-500">Task</span>
                <span class="font-mono truncate max-w-[150px]">{worker.task}</span>
              </div>
            </div>
            
            <!-- Activity Bar -->
            <div class="mt-4 h-1 bg-gray-700 rounded-full overflow-hidden">
              <div class="h-full bg-hpc-cyan animate-pulse" style="width: {worker.status === 'busy' ? '80%' : '5%'}"></div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Job Log -->
    <div class="bg-hpc-blue/30 border border-gray-800 rounded-xl p-6 h-[600px] flex flex-col">
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
