<script lang="ts">
  import { page } from '$app/stores';
  import { appAPI } from '$lib/api';
  import { createCommandStatusStore, createRecentActivityStore } from '$lib/stores';
  import type { Command } from '$lib/types';
  import { getCommandDescriptor, showCommandTitleWithMonospace } from '$lib/utils';
  import ActivityLight from './ActivityLight.svelte';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';

  export let command: Command;

  let commandStatus = createCommandStatusStore(command.id);
  let recentActivity = createRecentActivityStore(command.id);
  let currentCommandId = command.id;

  $: {
    if (command && currentCommandId !== command.id) {
      commandStatus = createCommandStatusStore(command.id);
      recentActivity = createRecentActivityStore(command.id);
      currentCommandId = command.id;
    }
  }

  $: selectedCommand = $page.data.command?.id;
</script>

<a
  class="flex border border-zinc-300 bg-zinc-50 hover:bg-zinc-100 transition-colors rounded-md items-center"
  class:font-mono={showCommandTitleWithMonospace(command)}
  class:selected={selectedCommand === command.id}
  class:deemphasize={selectedCommand && selectedCommand !== command.id}
  class:running={$commandStatus === 'Running'}
  class:stopping={$commandStatus === 'Stopping'}
  draggable
  on:dragstart
  on:dragend
  href={`/command/${command.id}`}
>
  <div class="flex flex-col flex-1 min-w-0 pl-4 gap-2">
    <span class="whitespace-nowrap text-ellipsis overflow-hidden">
      {command.name || getCommandDescriptor(command)}
    </span>
    <div class="flex flex-row gap-2">
      <ActivityLight
        green={$commandStatus === 'Running'}
        yellow={$commandStatus === 'Stopping'}
        red={$commandStatus === 'Stopped' &&
          command.lastRunResultType === 'exit' &&
          command.lastRunCode !== '0'}
      />
      <ActivityLight green={$recentActivity && $commandStatus === 'Running'} />
    </div>
  </div>

  <div class="m-4">
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
  </div>
</a>

<style lang="postcss">
  .selected {
    @apply border-zinc-400;
  }
</style>
