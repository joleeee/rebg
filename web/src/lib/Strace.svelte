<script>
    import { idxCss } from "./color";
    import { selectedAddress, selectedIdx } from "./stores";
    import { sendStore } from "./ws";

    export let tick = 0;
    export let data = "";

    let selectedI = null;
    selectedIdx.subscribe((x) => (selectedI = x));

    $: Tick = tick.toString().padStart(4, "\u00A0");
    $: highlightIdx = selectedI == tick;

    function click() {
        selectedIdx.set(tick);
        selectedAddress.set(-1);
        sendStore.set(JSON.stringify({ registers: tick }));
    }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- svelte-ignore a11y-click-events-have-key-events -->
<div on:click={click}>
    <span style={idxCss(highlightIdx)}>{Tick}</span>&nbsp;<span>{data}</span>
</div>

<style>
    div {
        font-family: monospace;
    }
</style>
