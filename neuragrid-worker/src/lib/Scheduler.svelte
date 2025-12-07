<script lang="ts">
    import { createEventDispatcher } from "svelte";

    // 7 days x 24 hours (true = active)
    export let schedule: boolean[][] = [];
    export let isOpen = false;

    const dispatch = createEventDispatcher();
    const days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    const hours = Array.from({ length: 24 }, (_, i) => i);

    let isDragging = false;
    let dragStartValue = true;

    function toggleSlot(d: number, h: number) {
        if (!schedule[d]) return; // Safety
        schedule[d][h] = !schedule[d][h];
        schedule = [...schedule]; // Trigger reactivity
    }

    function handleMouseDown(d: number, h: number) {
        isDragging = true;
        dragStartValue = !schedule[d][h]; // Value to apply (invert current)
        schedule[d][h] = dragStartValue;
        schedule = [...schedule];
    }

    function handleMouseEnter(d: number, h: number) {
        if (isDragging) {
            schedule[d][h] = dragStartValue;
            schedule = [...schedule];
        }
    }

    function handleMouseUp() {
        isDragging = false;
    }

    function save() {
        dispatch("save", schedule);
        isOpen = false;
    }
</script>

<svelte:window on:mouseup={handleMouseUp} />

{#if isOpen}
    <!-- Backdrop -->
    <div
        class="fixed inset-0 bg-black/80 flex items-center justify-center z-50 backdrop-blur-sm transition-opacity"
        on:click={() => (isOpen = false)}
        role="button"
        tabindex="0"
    >
        <!-- Modal -->
        <div
            class="bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl p-6 w-[95vw] max-w-4xl max-h-[90vh] flex flex-col"
            on:click|stopPropagation
            role="button"
            tabindex="0"
        >
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h2
                        class="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-purple-500"
                    >
                        Weekly Schedule
                    </h2>
                    <p class="text-sm text-gray-400 mt-1">
                        Click and drag to set active hours (Green = Active)
                    </p>
                </div>
                <button
                    on:click={() => (isOpen = false)}
                    class="text-gray-400 hover:text-white transition-colors"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>

            <!-- Scrollable Grid -->
            <div
                class="flex-grow overflow-auto border border-gray-800 rounded-lg p-4 bg-gray-950/50"
            >
                <div
                    class="grid grid-cols-[auto_repeat(24,_minmax(0,_1fr))] gap-1 min-w-[800px]"
                >
                    <!-- Header Row (Hours) -->
                    <div class="bg-transparent"></div>
                    <!-- Corner spacer -->
                    {#each hours as h}
                        <div
                            class="text-[10px] text-gray-500 text-center select-none pb-2"
                        >
                            {h}
                        </div>
                    {/each}

                    <!-- Rows (Days) -->
                    {#each days as day, d}
                        <!-- Day Label -->
                        <div
                            class="text-xs font-semibold text-gray-400 flex items-center pr-3 select-none"
                        >
                            {day}
                        </div>

                        <!-- Hour Slots -->
                        {#each hours as h}
                            <div
                                class="aspect-square rounded-sm border border-gray-800/50 transition-colors duration-150 cursor-pointer
                 {schedule[d] && schedule[d][h]
                                    ? 'bg-emerald-500/80 hover:bg-emerald-400'
                                    : 'bg-gray-800 hover:bg-gray-700'}"
                                on:mousedown={() => handleMouseDown(d, h)}
                                on:mouseenter={() => handleMouseEnter(d, h)}
                                role="button"
                                tabindex="0"
                            ></div>
                        {/each}
                    {/each}
                </div>
            </div>

            <!-- Footer Actions -->
            <div
                class="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-800"
            >
                <button
                    on:click={() => (isOpen = false)}
                    class="px-5 py-2 rounded-lg text-sm font-medium text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                >
                    Cancel
                </button>
                <button
                    on:click={save}
                    class="px-6 py-2 rounded-lg text-sm font-bold bg-gradient-to-r from-blue-500 to-purple-600 text-white hover:shadow-lg hover:shadow-purple-500/20 active:scale-95 transition-all"
                >
                    Save Schedule
                </button>
            </div>
        </div>
    </div>
{/if}
