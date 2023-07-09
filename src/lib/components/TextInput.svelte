<script lang="ts">
  export let id: string | undefined = undefined;
  export let value: string;
  export let placeholder: string;
  export let monospace = false
</script>

<input
  {id}
  bind:value
  on:blur
  {placeholder}
  class="bg-transparent block w-full focus:bg-zinc-50 transition-all h-12 focus:px-4 focus:outline-none font-sans placeholder:font-sans"
  class:font-mono={monospace}
/>
