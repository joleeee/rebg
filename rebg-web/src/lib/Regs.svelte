<script>
    import { onMount } from "svelte";
    import Step from "./Step.svelte";
    import Reg from "./Reg.svelte";

    export let regs = [
        ["rax", 1234],
        ["rsp", 133713371337],
    ];
</script>

<div class="outer">
    {#each regs as r}
        <Reg name={r[0]} value={r[1]} />
    {/each}
</div>

<style>
    .outer {
        padding: 0;
        margin: 0;
    }
</style>
