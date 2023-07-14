<script lang="ts">
  import '../app.css';
  import { WindowState } from '$lib/types';
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { appAPI, apiUrls } from '$lib/api';
  import Command from '$lib/components/Command.svelte';
  import Button from '$lib/components/Button.svelte';

  export let data: LayoutData;

  $: windowState = $page.data.windowState;

  $: isEditing = windowState === WindowState.Editing;

  $: {
    appAPI().setWindowState(windowState);
  }

  async function onAdd() {
    const command = await appAPI().addCommand({
      name: '',
      command: '',
    });
    console.log({ command });

    await goto(`/command/${command.id}`);
  }
</script>

<div class="app" class:editing={isEditing}>
  <div style="grid-area: command-top" class="flex flex-row items-center gap-4 px-4">
    <Button icon="plus" title="Add Command" on:click={onAdd} />
  </div>
  <div style="grid-area: command-main">
    {#each data.commands as command}
      <Command {command} />
    {/each}
  </div>
  {#if isEditing}
    <slot />
  {/if}
</div>

<style lang="postcss">
  .app {
    @apply w-screen h-screen grid;

    grid:
      'command-top edit-top' 80px
      'command-main edit-main' 1fr
      / 300px;
  }

  .editing {
    grid-template-columns: 300px 1fr;
  }
</style>
