<script lang="ts">
    import Value from "./Value.svelte";

    export let kind = "";
    export let adr: BigInt = 0n;
    export let value: BigInt = 0n;

    $: r = kind == "r";
    $: w = kind == "w";

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
    <Value {r} {w} value={adr} />&nbsp;{Arrow}&nbsp;<Value {value} />
</div>

<style>
    div {
        font-family: monospace;
    }
</style>
