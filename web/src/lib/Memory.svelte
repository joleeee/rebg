<script>
    import Value from "./Value.svelte";
    import { memoryStore } from "./ws";

    let data = [
        [0x500800, "0x4000801000a00000", "0x4000801000a00000"],
        [0x500810, "0x4000801000a00000", "0x4000801000a00000"],
        [0x500820, "0x4000801000a00000", "0x4000801000a00000"],
        [0x500830, "0x4000801000a00000", "0x4000801000a00000"],
        [0x500840, "0x4000801000a00000", "0x4000801000a00000"],
        [0x500850, "0x4000801000a00000", "0x4000801000a00000"],
        [0x500860, "0x4000801000a00000", "0x4000801000a00000"],
        [0x500870, "0x4000801000a00000", "0x4000801000a00000"],
    ];

    memoryStore.subscribe((x) => recv_mem(x));
    function recv_mem(mem) {
        if (mem === null) {
            return;
        }
        data = mem;
    }
</script>

<div>
    {#each data as row}
        <div>
            <!-- this is a really ugly hack to remove type errors -->
            <Value value={BigInt(row[0])} />
            {#each row.slice(1) as entry}
                &nbsp;<span>{entry}</span>
            {/each}
        </div>
    {/each}
</div>

<style>
    div {
        font-family: monospace;
    }
</style>
