<script>
  import MemOps from "./lib/MemOps.svelte";
  import Regs from "./lib/Regs.svelte";
  import Stepline from "./lib/Stepline.svelte";
  import Straceline from "./lib/Straceline.svelte";
</script>

<div class="timeline"><div class="padder">some list here</div></div>
<div class="stepline"><div class="padder"><Stepline /></div></div>
<div class="right">
  <div class="regs"><div class="padder"><Regs /></div></div>
  <div class="memops"><div class="padder"><MemOps /></div></div>
  <div class="strace"><div class="padder"><Straceline /></div></div>
</div>

<style>
  .padder {
    padding: 0.5em;
  }

  .timeline,
  .stepline,
  .right {
    padding: 0;
    height: 100vh;
    overflow-y: auto;
    flex-direction: column;
  }

  .timeline {
    text-align: center;
    color: white;
    background-color: blue;
  }

  .stepline {
    background-color: white;
    min-width: 40vw;
  }

  .right {
    min-width: 12em;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .regs {
    color: white;
    background-color: green;
  }

  .memops {
    color: white;
    background-color: maroon;
    flex: 1;
  }

  .strace {
    color: white;
    min-height: 4em;
    background-color: gray;
    flex: 1;
    overflow: scroll;
  }
</style>
