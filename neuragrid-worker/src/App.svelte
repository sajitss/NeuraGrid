<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import Scheduler from "./lib/Scheduler.svelte";

  let status = "Disconnected";
  let workerName = "Initializing...";
  let gpuUsage = 0;
  let earnings = 0.0;
  let logs: string[] = [];
  let unlisten: (() => void)[] = [];

  // Config state
  let config = {
    name: "",
    coordinator_url: "",
    silent_mode: false,
    schedule: Array(7).fill(Array(24).fill(true)),
  };

  async function loadConfig() {
    try {
      config = await invoke("get_config");
      addLog(`Config loaded. Silent Mode: ${config.silent_mode}`);
    } catch (e) {
      addLog(`Failed to load config: ${e}`);
    }
  }

  async function toggleSilentMode() {
    config.silent_mode = !config.silent_mode;
    try {
      await invoke("save_config", { newConfig: config });
      addLog(`Silent Mode set to ${config.silent_mode ? "Active" : "Off"}`);
    } catch (e) {
      addLog(`Failed to save config: ${e}`);
      config.silent_mode = !config.silent_mode; // Revert on failure
    }
  }

  let showScheduler = false;

  async function handleSaveSchedule(event: CustomEvent) {
    config.schedule = event.detail;
    try {
      await invoke("save_config", { newConfig: config });
      addLog("Schedule saved successfully.");
    } catch (e) {
      addLog(`Failed to save schedule: ${e}`);
    }
  }

  function addLog(msg: string) {
    logs = [...logs, `> ${msg}`];
    if (logs.length > 50) logs.shift();
  }

  let capabilities = [
    {
      code: "Pd",
      title: "People Detection",
      desc: "timestamps of person appearances, or True/False if any person present",
      style:
        "border-cyan-500 text-cyan-400 hover:bg-cyan-500/10 hover:shadow-[0_0_10px_rgba(6,182,212,0.5)]",
    },
    {
      code: "Pr",
      title: "Person Re‑Identification",
      desc: "timestamps where a specific person reappears across cameras",
      style:
        "border-purple-500 text-purple-400 hover:bg-purple-500/10 hover:shadow-[0_0_10px_rgba(168,85,247,0.5)]",
    },
    {
      code: "Ot",
      title: "Object Tracking",
      desc: "timestamps of object movement across frames",
      style:
        "border-emerald-500 text-emerald-400 hover:bg-emerald-500/10 hover:shadow-[0_0_10px_rgba(16,185,129,0.5)]",
    },
    {
      code: "Vd",
      title: "Vehicle Detection",
      desc: "timestamps of car appearances, or True/False if any car present",
      style:
        "border-orange-500 text-orange-400 hover:bg-orange-500/10 hover:shadow-[0_0_10px_rgba(249,115,22,0.5)]",
    },
    {
      code: "Lp",
      title: "License Plate Recognition",
      desc: "timestamps of a specific plate, or True/False if matched",
      style:
        "border-rose-500 text-rose-400 hover:bg-rose-500/10 hover:shadow-[0_0_10px_rgba(244,63,94,0.5)]",
    },
    {
      code: "Vs",
      title: "Video Search / Retrieval",
      desc: "timestamps of query match, or True/False if found",
      style:
        "border-fuchsia-500 text-fuchsia-400 hover:bg-fuchsia-500/10 hover:shadow-[0_0_10px_rgba(217,70,239,0.5)]",
    },
  ];

  onMount(async () => {
    addLog("System initialized.");
    await loadConfig();

    try {
      const unlistenStatus = await listen<string>(
        "connection-status",
        (event) => {
          if (status !== event.payload) {
            status = event.payload;
            addLog(`Connection status: ${status}`);
          }
        },
      );
      unlisten.push(unlistenStatus);

      const unlistenHardware = await listen<any>("hardware-info", (event) => {
        const info = event.payload;
        addLog(
          `Hardware detected: ${info.cpu_brand}, ${info.gpus.length} GPUs`,
        );
      });
      unlisten.push(unlistenHardware);

      const unlistenJob = await listen<string>("job-status", (event) => {
        addLog(event.payload);
      });
      unlisten.push(unlistenJob);

      const unlistenLog = await listen<string>("log-message", (event) => {
        addLog(event.payload);
      });
      unlisten.push(unlistenLog);

      const unlistenName = await listen<string>("worker-name", (event) => {
        workerName = event.payload;
      });
      unlisten.push(unlistenName);

      addLog("Setting up earnings listener...");
      const unlistenEarnings = await listen<string>(
        "earnings-update",
        (event) => {
          addLog(`Raw earnings event: ${event.payload}`);
          const val = parseFloat(event.payload);
          if (!isNaN(val)) {
            earnings = val;
            addLog(`Earnings updated: $${earnings.toFixed(2)}`);
          } else {
            addLog(`Failed to parse float from: ${event.payload}`);
          }
        },
      );
      unlisten.push(unlistenEarnings);
    } catch (e) {
      console.error("Failed to setup listeners", e);
      addLog(`Error: ${e}`);
    }
  });

  onDestroy(() => {
    unlisten.forEach((u) => u());
  });
</script>

<main
  class="container mx-auto p-4 h-full flex flex-col bg-gray-900 text-white relative"
>
  <header class="flex justify-between items-center mb-8">
    <div class="flex items-center gap-4">
      <img
        src="/logo.png"
        alt="NeuraGrid Logo"
        class="h-12 w-12 object-contain"
      />
      <div>
        <h1
          class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-purple-600"
        >
          NeuraGrid Worker
        </h1>
        <p class="text-sm text-gray-400 font-mono">@{workerName}</p>
      </div>
    </div>

    <div class="flex items-center gap-3">
      <!-- Schedule Button (Icon Only) -->
      <button
        on:click={() => (showScheduler = true)}
        title="Weekly Schedule"
        class="h-10 w-10 flex items-center justify-center rounded-lg bg-gray-800 border border-gray-700 hover:bg-gray-700 transition-colors text-gray-400 hover:text-white"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect><line
            x1="16"
            y1="2"
            x2="16"
            y2="6"
          ></line><line x1="8" y1="2" x2="8" y2="6"></line><line
            x1="3"
            y1="10"
            x2="21"
            y2="10"
          ></line></svg
        >
      </button>

      <!-- Silent Mode Toggle (Compact) -->
      <button
        on:click={toggleSilentMode}
        class="h-10 flex items-center gap-3 px-4 rounded-lg border transition-all duration-300 group
          {config.silent_mode
          ? 'bg-emerald-500/10 border-emerald-500/50 hover:bg-emerald-500/20'
          : 'bg-gray-800 border-gray-700 hover:bg-gray-750'}"
      >
        <span
          class="text-xs font-bold uppercase tracking-wide {config.silent_mode
            ? 'text-emerald-400'
            : 'text-gray-400'}"
        >
          {config.silent_mode ? "Active" : "Silent"}
        </span>

        <!-- Switch Graphic -->
        <div
          class="w-8 h-4 rounded-full relative transition-colors duration-300 {config.silent_mode
            ? 'bg-emerald-500'
            : 'bg-gray-600'}"
        >
          <div
            class="absolute top-0.5 left-0.5 w-3 h-3 rounded-full bg-white transition-transform duration-300 {config.silent_mode
              ? 'translate-x-4'
              : 'translate-x-0'}"
          ></div>
        </div>
      </button>

      <div
        class="h-10 flex items-center px-4 rounded-lg bg-gray-800 border border-gray-700 text-sm"
      >
        Status: <span class="text-green-400 font-semibold ml-1">{status}</span>
      </div>
    </div>
  </header>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <div class="bg-gray-800 rounded-xl p-6 border border-gray-700 shadow-lg">
      <h2 class="text-xl font-semibold mb-4 text-gray-200">GPU Performance</h2>
      <div class="flex items-end gap-2 mb-2">
        <span class="text-4xl font-bold text-blue-400">{gpuUsage}%</span>
        <span class="text-gray-400 mb-1">utilization</span>
      </div>
      <div class="w-full bg-gray-700 rounded-full h-2.5">
        <div
          class="bg-blue-500 h-2.5 rounded-full"
          style="width: {gpuUsage}%"
        ></div>
      </div>
    </div>

    <div class="bg-gray-800 rounded-xl p-6 border border-gray-700 shadow-lg">
      <h2 class="text-xl font-semibold mb-4 text-gray-200">Earnings</h2>
      <div class="flex items-end gap-2">
        <span class="text-4xl font-bold text-green-400"
          >${earnings.toFixed(2)}</span
        >
        <span class="text-gray-400 mb-1">today</span>
      </div>
    </div>

    <!-- Capabilities Card -->
    <div class="bg-gray-800 rounded-xl p-6 border border-gray-700 shadow-lg">
      <h2 class="text-xl font-semibold mb-4 text-gray-200">Capabilities</h2>
      <div class="flex flex-wrap gap-2">
        {#each capabilities as cap}
          <div class="group relative">
            <div
              class="w-10 h-10 flex items-center justify-center bg-gray-800/50 rounded-lg text-sm font-bold cursor-help transition-all duration-300 border {cap.style}"
            >
              {cap.code}
            </div>
            <!-- Tooltip -->
            <div
              class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 w-48 p-2 bg-gray-900 border border-gray-600 rounded shadow-xl text-xs text-gray-300 z-10 hidden group-hover:block pointer-events-none"
            >
              <strong class="block text-blue-400 mb-1">{cap.title}</strong>
              {cap.desc}
              <div
                class="absolute top-full left-1/2 transform -translate-x-1/2 -mt-1 border-4 border-transparent border-t-gray-600"
              ></div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <div
    class="mt-8 bg-gray-800 rounded-xl p-6 border border-gray-700 shadow-lg flex-grow"
  >
    <h2 class="text-xl font-semibold mb-4 text-gray-200">Activity Log</h2>
    <div
      class="h-48 overflow-y-auto font-mono text-sm text-gray-400 bg-gray-900 p-4 rounded border border-gray-700"
    >
      {#each logs as log}
        <p>{log}</p>
      {/each}
    </div>
  </div>

  <Scheduler
    bind:isOpen={showScheduler}
    schedule={config.schedule}
    on:save={handleSaveSchedule}
  />
</main>
