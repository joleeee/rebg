<script>
    import Strace from "./Strace.svelte";
    import { straceStore } from "./ws";

    export let calls = [[2, "write(...)"], [28, "exit(0)"]];

    straceStore.subscribe(recv_strace);
    function recv_strace(strace) {
        if (strace === null) {
            return;
        }

        calls = strace;
    }
</script>

<div>
    {#each calls as c}
        <Strace tick={c[0]} data={c[1]} />
    {/each}
</div>
