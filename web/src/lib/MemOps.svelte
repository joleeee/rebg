<script>
    import MemOp from "./MemOp.svelte";
    import { memOpsStore } from "./ws";

    export let reads = [];
    export let writes = [];

    memOpsStore.subscribe(recv_memOps);

    function recv_memOps(memOps) {
        if (memOps === null) {
            return;
        }

        reads = memOps.r;
        writes = memOps.w;
    }
</script>

<div>
    {#each reads as r}
        <MemOp kind="r" adr={r[0]} value={r[1]} />
    {/each}
    {#each writes as w}
        <MemOp kind="w" adr={w[0]} value={w[1]} />
    {/each}
</div>
