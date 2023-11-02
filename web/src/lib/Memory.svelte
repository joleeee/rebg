<script>
    let data = [
        ["0x500800", "0x4000801000a00000", "0x4000801000a00000"],
        ["0x500810", "0x4000801000a00000", "0x4000801000a00000"],
    ];

    const height = 8;
    const width = 2;
</script>

<div>
    {#each data as row}
        <div>
            <span>{row[0]}:</span>&nbsp;
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
