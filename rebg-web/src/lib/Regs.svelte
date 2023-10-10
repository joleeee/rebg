<script>
    import Reg from "./Reg.svelte";
    import { registerStore, connectedStore } from "./ws";

    export let regs = [
        ["rax", 1234],
        ["rsp", 133713371337],
    ];

    let connected = false;
    connectedStore.subscribe((x) => {
        connected = x;
        if (connected) {
            regs = [];
        }
    });

    registerStore.subscribe(recv_registers);

    function recv_registers(registers) {
        if (registers === null) {
            return;
        }
        let rs = registers.registers;
        regs = rs;
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
