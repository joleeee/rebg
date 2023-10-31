<script>
    import { createEventDispatcher } from "svelte";
    import { rwCssEntry } from "./color";
    const dispatch = createEventDispatcher();

    export let name, value, mod;
    $: Name = name.padStart(3, "\u00A0");
    $: Value = "0x" + parseInt(value).toString(16);
    $: r = mod && mod.includes("r");
    $: w = mod && mod.includes("w");

    function click() {
        dispatch("selected", { name: name, value: value });
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div on:click={click}>
    <span class="name">{Name}</span>
    <span class="val" style={rwCssEntry(r, w)}>{Value}</span>
</div>

<style>
    div {
        font-family: monospace;
    }
    .name {
        color: black;
    }
    .val {
        color: white;
    }
</style>
