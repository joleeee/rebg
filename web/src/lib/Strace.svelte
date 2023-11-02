<script>
    import { idxCss } from "./color";
    import { selectedIdx } from "./stores";

    export let tick = 0;
    export let data = "";

    let selectedI = null;
    selectedIdx.subscribe((x) => (selectedI = x));

    $: Tick = tick.toString().padStart(4, "\u00A0");
    $: highlightIdx = selectedI == tick;
</script>

<div>
    <span style={idxCss(highlightIdx)}>{Tick}</span>&nbsp;<span>{data}</span>
</div>

<style>
    div {
        font-family: monospace;
    }
</style>
