<script lang="ts">
  import type { PageData } from './$types';

  import { goto } from '$app/navigation';

  import { appAPI } from '$lib/api';
  import { getCommandDescriptor, showCommandTitleWithMonospace } from '$lib/utils';

  import TextInput from '$lib/components/TextInput.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { getLogLinesStore } from './utils';

  import Console from '$lib/components/Console.svelte';

  export let data: PageData;

  $: command = data.command;

  let logLines = getLogLinesStore(data.command.id, data.initialCommandLogLines);
  let currentCommandId = data.command.id;

  $: {
    if (command && currentCommandId !== command.id) {
      logLines = getLogLinesStore(command.id, data.initialCommandLogLines);
      currentCommandId = command.id;
    }
  }

  $: statusText = data.processStatus;

  async function saveChanges() {
    const { name, command: cmd, cwd } = command;
    await appAPI().updateCommand(command.id, { name, command: cmd, cwd });
  }
</script>

<div style="grid-area: edit-top" class="ui-row">
  <Button icon="arrowLeft" title="Back" href="/" />
  <div class="flex-1">
    <TextInput
      bind:value={command.name}
      on:blur={saveChanges}
      placeholder={getCommandDescriptor(command)}
      monospace={showCommandTitleWithMonospace(command)}
    />
  </div>
  <Button
    icon="delete"
    title="Delete"
    on:click={async () => {
      await appAPI().deleteCommand(command.id);
      await goto('/');
    }}
  />
</div>

<div style="grid-area: edit-main" class="flex flex-col gap-4 overflow-hidden">
  <div class="body-ui-row">
    <div class="icon-cont">
      <Icon icon="folder" title="Directory" />
    </div>
    <div class="flex-1">
      <TextInput
        bind:value={command.cwd}
        on:blur={saveChanges}
        placeholder={'Working directory'}
        monospace
      />
    </div>
  </div>
  <div class="body-ui-row">
    <div class="icon-cont">
      <Icon icon="console" title="Command" />
    </div>
    <div class="flex-1">
      <TextInput
        bind:value={command.command}
        on:blur={saveChanges}
        placeholder={'Command'}
        monospace
      />
    </div>
  </div>

  <div class="body-ui-row">
    <p class="flex-1">
      Status: {statusText}
    </p>
    {#if data.processStatus === 'Stopped'}
      <Button
        icon="play"
        title="Start"
        on:click={async () => {
          await appAPI().runProcess(command.id);
        }}
      />
    {:else}
      <Button
        icon="stop"
        title="Stop"
        on:click={async () => {
          await appAPI().killProcess(command.id);
        }}
      />
    {/if}
  </div>

  {#key command.id}
    <Console logLines={$logLines} on:load-more={() => logLines.loadMore()} />
  {/key}
</div>

<style lang="postcss">
  .ui-row {
    @apply flex flex-row items-center gap-4 px-4;
  }

  .body-ui-row {
    @apply ui-row h-12;
  }

  .icon-cont {
    @apply w-12 h-12 p-2;
  }
</style>
