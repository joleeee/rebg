<script>
    export let depth, idx, adr, asm;

    $: indent = "\u00A0".repeat(depth); // nbsp
    $: index = parseInt(idx).toString().padStart(4, "\u00A0");
    $: address = "0x" + parseInt(adr).toString(16);
</script>

<div>
    <span class="i">{indent}{index}</span>
    <span class="adr">{address}</span>
    <span>{asm}</span>
</div>

<style>
    div {
        font-family: monospace;
    }
    .i {
        color: blue;
        background-color: white;
    }
    .adr {
        color: red;
    }
</style>
