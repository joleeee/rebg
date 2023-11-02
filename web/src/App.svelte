<script>
  import MemOps from "./lib/MemOps.svelte";
    import Memory from "./lib/Memory.svelte";
  import Regs from "./lib/Regs.svelte";
  import Stepline from "./lib/Stepline.svelte";
  import Straceline from "./lib/Straceline.svelte";
</script>

<div class="timeline"><div class="padder">some list here</div></div>
<div class="middle">
  <div class="stepline"><div class="padder"><Stepline /></div></div>
  <div class="memory"><div class="padder"><Memory /></div></div>
</div>
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
  .middle,
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

  .middle {
    min-width: 12em;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .stepline {
    background-color: white;
    min-width: 40vw;
    overflow: auto;
  }

  .memory {
    background-color: gainsboro;
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
    overflow: auto;
  }

  .strace {
    color: white;
    min-height: 4em;
    background-color: gray;
    flex: 1;
    overflow: auto;
  }
</style>
