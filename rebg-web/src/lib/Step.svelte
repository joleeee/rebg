<script>
    import { selectedAddress } from "./stores.js";

    export let depth, idx, adr, asm;

    $: indent = "\u00A0".repeat(depth); // nbsp
    $: index = parseInt(idx).toString().padStart(4, "\u00A0");
    $: address = "0x" + parseInt(adr).toString(16);

    function click() {
        selectedAddress.set(address);
    }

    let selected = null;
    $: highlight = selected == address;

    selectedAddress.subscribe((x) => (selected = x));
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div on:click={click}>
    <span class="i">{indent}{index}</span>
    <span class="adr" style={highlight ? "background-color: #ff000088;" : ""}
        >{address}</span
    >
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
