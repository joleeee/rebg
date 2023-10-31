<script>
    import { rwCssEntry } from "./color";

    export let kind = "";
    export let adr = 0;
    export let value = 0;

    $: r = kind == "r";
    $: w = kind == "w";

    $: Address = "0x" + adr.toString(16);
    $: Value = "0x" + value.toString(16);
    $: Arrow = arrow(kind);

    function arrow(k) {
        if (k === "r") {
            return "->";
        } else if (k == "w") {
            return "<-";
        } else {
            return "??";
        }
    }
</script>

<div>
    <span style={rwCssEntry(r, w)}>{Address}</span>&nbsp;{Arrow}&nbsp;<span
        >{Value}</span
    >
</div>

<style>
    div {
        font-family: monospace;
    }
</style>
