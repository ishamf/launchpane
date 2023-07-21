<script>
  export let green = false;
  export let yellow = false;
  export let red = false;
</script>

<div class="w-4 h-2 rounded-sm transition-colors bg-zinc-200"
  class:green
  class:yellow
  class:red
/>

<style lang="postcss">
  .green {
    @apply bg-lime-400;
  }

  .yellow {
    @apply bg-yellow-400;
  }

  .red {
    @apply bg-red-400;
  }
</style>
