<script>
    import Value from "./Value.svelte";
    import { selectedAddress } from "./stores";
    import { memOpsStore, memoryStore } from "./ws";

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

    let r_adrs = [];
    let w_adrs = [];

    memoryStore.subscribe(recv_mem);
    memOpsStore.subscribe(recv_memOps);

    function recv_mem(mem) {
        if (mem === null) {
            return;
        }
        data = mem;
    }

    function recv_memOps(ops) {
        if (ops === null) {
            return;
        }
        r_adrs = ops.r.map((a) => a[0]);
        w_adrs = ops.w.map((a) => a[0]);
    }

    function key_press(event) {
        if (event.key == "g") {
            const a = prompt("Address?");
            if (a === null) {
                return;
            }
            const v = BigInt(a); // handles optional 0x prefix too
            selectedAddress.set(Number(v));
        }
    }
</script>

<!-- <svelte:window on:keypress={key_press} /> -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
<div on:keypress={key_press} tabindex="0">
    {#each data as row}
        <div>
            <!-- this is a really ugly hack to remove type errors -->
            <Value
                value={BigInt(row[0])}
                r={r_adrs.includes(row[0])}
                w={w_adrs.includes(row[0])}
            />
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
