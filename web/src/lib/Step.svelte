<script>
    import { createEventDispatcher } from "svelte";
    import { selectedAddress, selectedIdx, showSymbols } from "./stores.js";
    import { adrCss, idxCss } from "./color.js";
    const dispatch = createEventDispatcher();

    /** @type { Element } */
    let thisStep;

    let show;
    showSymbols.subscribe((x) => (show = x));

    export let depth, idx, adr, asm, symbol;
    $: Indent = "\u00A0".repeat(depth); // nbsp
    $: Index = parseInt(idx).toString().padStart(4, "\u00A0");
    $: Address = (show && symbol) || "0x" + parseInt(adr).toString(16);

    function click() {
        dispatch("selected", { index: idx, address: adr });
        selectedAddress.set(adr);
        selectedIdx.set(idx);
    }

    let selectedA = null;
    let selectedI = null;
    selectedAddress.subscribe((x) => (selectedA = x));
    selectedIdx.subscribe((x) => (selectedI = x));
    $: highlightAdr = selectedA == adr;
    $: highlightIdx = selectedI == idx;

    // if it's in view, scroll to it (nice for keybinds!)
    $: if (highlightIdx) {
        thisStep.scrollIntoView({ block: "nearest" });
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div on:click={click} bind:this={thisStep}>
    <span class="idx" style={idxCss(highlightIdx)}>{Indent}{Index}</span>
    <span class="adr" style={adrCss(highlightAdr)}>{Address}</span>
    <span>{asm}</span>
</div>

<style>
    div {
        font-family: monospace;
    }
    .idx {
        color: var(--step-color);
    }
    .adr {
        color: var(--adr-color);
    }
</style>
