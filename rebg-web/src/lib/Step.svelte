<script>
    import { createEventDispatcher } from "svelte";
    import { selectedAddress, selectedIdx } from "./stores.js";
    const dispatch = createEventDispatcher();
    const addressSelected = "#ff000060";
    const idxSelected = "#ff0000c0";
    /** @type { Element } */
    let thisStep;

    export let depth, idx, adr, asm, symbol;
    $: Indent = "\u00A0".repeat(depth); // nbsp
    $: Index = parseInt(idx).toString().padStart(4, "\u00A0");
    $: Address = symbol || "0x" + parseInt(adr).toString(16);

    function click() {
        dispatch("selected", { index: idx, address: adr });
        selectedAddress.set(Address);
        selectedIdx.set(Index);
    }

    let selectedA = null;
    let selectedI = null;
    selectedAddress.subscribe((x) => (selectedA = x));
    selectedIdx.subscribe((x) => (selectedI = x));
    $: highlightAdr = selectedA == Address;
    $: highlightIdx = selectedI == idx;

    // if it's in view, scroll to it (nice for keybinds!)
    $: if (highlightIdx) {
        thisStep.scrollIntoView({ block: "nearest" });
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div on:click={click} bind:this={thisStep}>
    <span
        class="idx"
        style={highlightIdx ? `background-color: ${idxSelected};` : ""}
        >{Indent}{Index}</span
    >
    <span
        class="adr"
        style={highlightAdr ? `background-color: ${addressSelected};` : ""}
        >{Address}</span
    >
    <span>{asm}</span>
</div>

<style>
    div {
        font-family: monospace;
    }
    .idx {
        color: blue;
    }
    .adr {
        color: red;
    }
</style>
