<script>
    import { memoryStore } from "./ws";

    let data = [
        ["0x500800", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500810", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500820", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500830", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500840", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500850", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500860", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500870", "0x4000801000a00000", "0x4000801000a00000"],
    ];

    memoryStore.subscribe((x) => recv_mem(x));
    function recv_mem(mem) {
        if (mem === null) {
            return;
        }
        data = mem;
    }

    function hexify(value) {
        return "0x" + parseInt(value).toString(16);
    }

    const height = 8;
    const width = 2;
</script>

<div>
    {#each data as row}
        <div>
            <span>{hexify(row[0])}:</span>&nbsp;
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
