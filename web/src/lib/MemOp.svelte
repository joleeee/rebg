<script>
    export let kind = "";
    export let adr = 0;
    export let value = 0;

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
    <span>{Address}</span>
    {Arrow} <span>{Value}</span>
</div>
