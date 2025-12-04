<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  
  let status = "Disconnected";
  let gpuUsage = 0;
  let earnings = 0.00;
  let logs: string[] = [];
  let unlisten: (() => void)[] = [];

  function addLog(msg: string) {
    logs = [...logs, `> ${msg}`];
    if (logs.length > 50) logs.shift();
  }

  onMount(async () => {
    addLog("System initialized.");
    
    try {
      const unlistenStatus = await listen<string>('connection-status', (event) => {
        status = event.payload;
        addLog(`Connection status: ${status}`);
      });
      unlisten.push(unlistenStatus);

      const unlistenHardware = await listen<any>('hardware-info', (event) => {
        const info = event.payload;
        addLog(`Hardware detected: ${info.cpu_brand}, ${info.gpus.length} GPUs`);
      });
      unlisten.push(unlistenHardware);

      const unlistenJob = await listen<string>('job-status', (event) => {
        addLog(event.payload);
      });
      unlisten.push(unlistenJob);
    } catch (e) {
      console.error("Failed to setup listeners", e);
      addLog(`Error: ${e}`);
    }
  });

  onDestroy(() => {
    unlisten.forEach(u => u());
  });
</script>

<main class="container mx-auto p-4 h-full flex flex-col bg-gray-900 text-white">
  <header class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-purple-600">
      NeuraGrid Worker
    </h1>
    <div class="flex gap-2">
       <div class="px-3 py-1 rounded-full bg-gray-800 border border-gray-700 text-sm">
         Status: <span class="text-green-400 font-semibold">{status}</span>
       </div>
    </div>
  </header>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <div class="bg-gray-800 rounded-xl p-6 border border-gray-700 shadow-lg">
      <h2 class="text-xl font-semibold mb-4 text-gray-200">GPU Performance</h2>
      <div class="flex items-end gap-2 mb-2">
        <span class="text-4xl font-bold text-blue-400">{gpuUsage}%</span>
        <span class="text-gray-400 mb-1">utilization</span>
      </div>
      <div class="w-full bg-gray-700 rounded-full h-2.5">
        <div class="bg-blue-500 h-2.5 rounded-full" style="width: {gpuUsage}%"></div>
      </div>
    </div>

    <div class="bg-gray-800 rounded-xl p-6 border border-gray-700 shadow-lg">
      <h2 class="text-xl font-semibold mb-4 text-gray-200">Earnings</h2>
      <div class="flex items-end gap-2">
        <span class="text-4xl font-bold text-green-400">${earnings.toFixed(2)}</span>
        <span class="text-gray-400 mb-1">today</span>
      </div>
    </div>
  </div>

  <div class="mt-8 bg-gray-800 rounded-xl p-6 border border-gray-700 shadow-lg flex-grow">
    <h2 class="text-xl font-semibold mb-4 text-gray-200">Activity Log</h2>
    <div class="h-48 overflow-y-auto font-mono text-sm text-gray-400 bg-gray-900 p-4 rounded border border-gray-700">
      {#each logs as log}
        <p>{log}</p>
      {/each}
    </div>
  </div>
</main>
