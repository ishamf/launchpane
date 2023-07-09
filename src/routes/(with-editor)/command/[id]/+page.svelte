<script lang="ts">
  import type { PageData } from './$types';

  import { goto, invalidate } from '$app/navigation';

  import { appAPI, apiUrls } from '$lib/api';
  import { getCommandDescriptor, showCommandTitleWithMonospace } from '$lib/utils';

  import TextInput from '$lib/components/TextInput.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';

  export let data: PageData;

  $: command = data.command;

  async function saveChanges() {
    await appAPI().updateCommand(command.id, command);
    invalidate(apiUrls.getCommands());
    invalidate(apiUrls.getCommand(command.id));
  }
</script>

<div style="grid-area: edit-top" class="flex flex-row items-center gap-4 px-4">
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
      await invalidate(apiUrls.getCommands());
      await goto('/');
    }}
  />
</div>

<div style="grid-area: edit-main" class="flex flex-col gap-4">
  <div class="flex flex-row items-center gap-4 px-4 h-12">
    <div class="w-12 h-12 p-2">
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
  <div class="flex flex-row items-center gap-4 px-4 h-12">
    <div class="w-12 h-12 p-2">
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
</div>
