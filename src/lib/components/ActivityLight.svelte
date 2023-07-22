<script lang="ts">
  export let green = false;
  export let yellow = false;
  export let red = false;

  export let transition = false;

  export let text: string;
</script>

<div
  class="flex items-center justify-center w-4 h-2 rounded-sm bg-zinc-200"
  class:green
  class:yellow
  class:red
  class:transition-colors={transition}
>
  <span
    class="block text-[6px] leading-[6px] text-zinc-400"
    class:green
    class:yellow
    class:red
    class:transition-colors={transition}>{text}</span
  >
</div>

<style lang="postcss">
  div.green {
    @apply bg-lime-400;
  }

  div.yellow {
    @apply bg-yellow-400;
  }

  div.red {
    @apply bg-red-400;
  }

  span.green {
    @apply text-lime-600;
  }

  span.yellow {
    @apply text-yellow-600;
  }

  span.red {
    @apply text-red-600;
  }
</style>
