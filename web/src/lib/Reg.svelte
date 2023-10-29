<script>
    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher();
    const modTextColor = "black";

    // basically stolen from qira
    const writeFg = "#FFFF00"; // bright yellow
    const readFg = "#888800"; // dark yellow
    const bothFg = "#CCAA00";

    export let name, value, mod;
    $: Name = name.padStart(3, "\u00A0");
    $: Value = "0x" + parseInt(value).toString(16);
    let BackgroundL = "";
    let BackgroundR = "";
    $: {
        let r = mod && mod.includes("r");
        let w = mod && mod.includes("w");
        if (r && w) {
            BackgroundL = readFg;
            BackgroundR = writeFg;
        } else if (r) {
            BackgroundL = readFg;
            BackgroundR = readFg;
        } else if (w) {
            BackgroundL = writeFg;
            BackgroundR = writeFg;
        } else {
            BackgroundL = "";
            BackgroundR = "";
        }
    }
    $: Color =
        (mod && mod.includes("w")) || (mod && mod.includes("r"))
            ? modTextColor
            : "";

    function click() {
        dispatch("selected", { name: name, value: value });
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div on:click={click}>
    <span class="name">{Name}</span>
    <span
        class="val"
        style={BackgroundL && BackgroundR
            ? `background: linear-gradient(90deg, ${BackgroundL} 15%, ${BackgroundR} 85%); color: ${Color}`
            : ""}>{Value}</span
    >
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
