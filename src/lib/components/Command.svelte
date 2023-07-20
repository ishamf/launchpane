<script lang="ts">
  import { page } from '$app/stores';
  import { appAPI } from '$lib/api';
  import { createCommandStatusStore } from '$lib/stores';
  import type { Command } from '$lib/types';
  import { getCommandDescriptor, showCommandTitleWithMonospace } from '$lib/utils';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';

  export let command: Command;

  let commandStatus = createCommandStatusStore(command.id);
  let currentCommandId = command.id;

  $: {
    if (command && currentCommandId !== command.id) {
      commandStatus = createCommandStatusStore(command.id);
      currentCommandId = command.id;
    }
  }

  $: selectedCommand = $page.data.command?.id;
</script>

<a
  class="flex p-4 m-4 border border-zinc-300 bg-zinc-50 hover:bg-zinc-100 transition-colors rounded-md items-center gap-4"
  class:font-mono={showCommandTitleWithMonospace(command)}
  class:selected={selectedCommand === command.id}
  class:deemphasize={selectedCommand && selectedCommand !== command.id}
  class:running={$commandStatus === 'Running'}
  class:stopping={$commandStatus === 'Stopping'}
  href={`/command/${command.id}`}
>
  <span class="whitespace-nowrap text-ellipsis overflow-hidden flex-1 min-w-0">
    {command.name || getCommandDescriptor(command)}
  </span>

  {#if $commandStatus === 'Stopped'}
    <Button
      small
      altColor
      icon="play"
      title="Start"
      on:click={async (e) => {
        e.preventDefault();
        await appAPI().runProcess(command.id);
      }}
    />
  {:else}
    <Button
      small
      altColor
      icon="stop"
      title="Stop"
      on:click={async (e) => {
        e.preventDefault();
        await appAPI().killProcess(command.id);
      }}
    />
  {/if}
</a>

<style lang="postcss">
  .selected {
    @apply border-zinc-500
  }

  .running {
    @apply bg-green-50 hover:bg-green-100;
  }

  .running.selected {
    @apply border-green-500;
  }

  .stopping {
    @apply bg-yellow-50 hover:bg-yellow-100;
  }

  .stopping.selected {
    @apply border-yellow-500;
  }
</style>
