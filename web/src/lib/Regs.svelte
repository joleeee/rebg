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
    <div class="inner">
        {#each regs.slice(0, Math.ceil(regs.length / 2)) as r}
            <Reg name={r[0]} value={r[1]} mod={r[2]} />
        {/each}
    </div>
    <div class="inner">
        {#each regs.slice(Math.ceil(regs.length / 2)) as r}
            <Reg name={r[0]} value={r[1]} mod={r[2]} />
        {/each}
    </div>
</div>

<style>
    .outer {
        padding: 0;
        margin: 0;
        display: flex;
    }
    .inner {
        /* 
           1em is approx 2 chars
           value: 64bit = 8 byte = 16 letters
           prefix: 2 letters
           name: 3 letters
           sep: 1 letter
           sum: 22 letters
        */
        min-width: 11em;
    }
</style>
