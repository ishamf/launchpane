<script lang="ts">
  import '../app.css';
  import { WindowState } from '$lib/types';
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { appAPI } from '$lib/api';

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
      cwd: '',
    });

    goto(`/command/${command.id}`);
  }
</script>

<div class="app" class:editing={isEditing}>
  <div style="grid-area: command-main">
    {#each data.commands as command}
      <a class="block p-4 m-4 bg-zinc-200 hover:bg-zinc-300" href={`/command/${command.id}`}
        >{command.name}</a
      >
    {/each}
    <button class="block p-4 m-4 bg-zinc-200 hover:bg-zinc-300" on:click={onAdd}>Add Command</button
    >
  </div>
  {#if isEditing}
    <slot />
  {/if}
</div>

<style lang="postcss">
  .app {
    @apply w-full h-full grid;

    grid:
      'command-top edit-top' 80px
      'command-main edit-main' auto
      / 300px;
  }

  .editing {
    grid-template-columns: 300px auto;
  }

  .command-bar {
  }

  .edit-space {
    grid-column: 2;
    grid-row: 1 / span 2;
  }
</style>
