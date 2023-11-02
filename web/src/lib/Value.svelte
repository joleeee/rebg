<script lang="ts">
    import { rwCssEntry } from "./color";

    export let value: BigInt = 0n;
    export let r = false;
    export let w = false;

    $: Value = "0x" + value.toString(16);
</script>

<span style={rwCssEntry(r, w)}>
    {Value}
</span>

<style>
    span {
        font-family: monospace;
    }
</style>
