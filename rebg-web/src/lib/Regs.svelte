<script>
    import Reg from "./Reg.svelte";
    import { registerStore, stepStore } from "./ws";

    export let regs = [
        ["rax", 1234],
        ["rsp", 133713371337],
    ];

    registerStore.subscribe(recv_registers);

    function recv_registers(registers) {
        if (registers === null) {
            return;
        }
        let index = registers.idx;
        let regs = registers.registers;
        console.log(index, regs);
    }
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
