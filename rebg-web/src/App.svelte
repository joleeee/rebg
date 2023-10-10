<script>
  import Regs from "./lib/Regs.svelte";
  import Stepline from "./lib/Stepline.svelte";
</script>

<div class="timeline"><div class="padder">some list here</div></div>
<div class="stepline"><div class="padder"><Stepline /></div></div>
<div class="regs"><div class="padder"><Regs /></div></div>

<style>
  .padder {
    padding: 0.5em;
  }

  .timeline,
  .stepline,
  .regs {
    padding: 0;
    height: 100vh;
    overflow-y: auto;
  }

  .timeline {
    text-align: center;
    color: white;
    background-color: blue;
  }

  .stepline {
    background-color: white;
    flex: 1;
  }

  .regs {
    color: white;
    background-color: green;
    min-width: 12em;
  }
</style>
