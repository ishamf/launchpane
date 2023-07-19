<script lang="ts">
  import { page } from '$app/stores';
  import type { Command } from '$lib/types';
  import { getCommandDescriptor, showCommandTitleWithMonospace } from '$lib/utils';

  export let command: Command;

  $: selectedCommand = $page.data.command?.id;
</script>

<div>
  <a
    class="block p-4 m-4 border border-zinc-300 bg-zinc-50 hover:bg-zinc-100 transition-colors rounded-md"
    class:font-mono={showCommandTitleWithMonospace(command)}
    class:active={selectedCommand === command.id}
    href={`/command/${command.id}`}
  >
    {command.name || getCommandDescriptor(command)}
  </a>
</div>

<style lang="postcss">
  .active {
    @apply bg-white border-zinc-500;
  }
</style>
