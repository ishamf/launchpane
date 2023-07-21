<script lang="ts">
  import '../app.css';
  import { WindowState } from '$lib/types';
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { appAPI } from '$lib/api';
  import Command from '$lib/components/Command.svelte';
  import Button from '$lib/components/Button.svelte';

  export let data: LayoutData;

  $: windowState = $page.data.windowState;

  $: isEditing = windowState === WindowState.Editing;

  $: {
    if (isEditing) {
      appAPI().setWindowSize(1380, 650);
    } else {
      appAPI().setWindowSize(400, 650);
    }
  }

  let draggingId: number | null = null;
  let currentDraggedOverComponent: string | null = null;
  let currentHoveredDropTargets: (number | undefined)[] = [];

  async function onAdd() {
    const command = await appAPI().createCommand();

    await goto(`/command/${command.id}`);
  }
</script>

<div class="app" class:editing={isEditing}>
  <div style="grid-area: command-top" class="flex flex-row items-center px-4">
    <Button icon="plus" title="Add Command" on:click={onAdd} />
  </div>
  <div style="grid-area: command-main" class="overflow-auto">
    {#each data.commands as command, commandIndex}
      <div class="relative py-2 px-4">
        <div
          class="transition-transform"
          class:-translate-y-2={currentHoveredDropTargets[0] === command.id}
          class:translate-y-2={currentHoveredDropTargets[1] === command.id}
        >
          <Command
            {command}
            on:dragstart={(e) => {
              if (e.dataTransfer) {
                e.dataTransfer.effectAllowed = 'move';
                e.dataTransfer.dropEffect = 'move';
                e.dataTransfer.setData('text/plain', `${command.id}`);
              }

              draggingId = command.id;
            }}
            on:dragend={() => {
              draggingId = null;
              currentHoveredDropTargets = [];
            }}
          />
        </div>

        {#if draggingId && draggingId !== command.id && data.commands[commandIndex - 1]?.id !== draggingId}
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div
            class="absolute top-0 bottom-1/2 left-0 right-0"
            on:dragover={(e) => {
              e.preventDefault();
            }}
            on:dragenter={(e) => {
              currentDraggedOverComponent = `${command.id}-top`;
              currentHoveredDropTargets = [data.commands[commandIndex - 1]?.id, command.id];
            }}
            on:dragleave={(e) => {
              if (currentDraggedOverComponent === `${command.id}-top`) {
                currentDraggedOverComponent = null;
                currentHoveredDropTargets = [];
              }
            }}
            on:drop={(e) => {
              e.preventDefault();

              const commandIdStr = e.dataTransfer?.getData('text/plain');

              if (commandIdStr) {
                const droppedCommandId = parseInt(commandIdStr, 10);
                appAPI().moveCommandBetween(
                  droppedCommandId,
                  data.commands[commandIndex - 1]?.id || null,
                  command.id,
                );
              }
            }}
          />
        {/if}
        {#if draggingId && draggingId !== command.id && data.commands[commandIndex + 1]?.id !== draggingId}
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div
            class="absolute top-1/2 bottom-0 left-0 right-0"
            on:dragover={(e) => {
              e.preventDefault();
            }}
            on:dragenter={(e) => {
              currentDraggedOverComponent = `${command.id}-bottom`;
              currentHoveredDropTargets = [command.id, data.commands[commandIndex + 1]?.id];
            }}
            on:dragleave={(e) => {
              if (currentDraggedOverComponent === `${command.id}-bottom`) {
                currentDraggedOverComponent = null;
                currentHoveredDropTargets = [];
              }
            }}
            on:drop={(e) => {
              e.preventDefault();

              const commandIdStr = e.dataTransfer?.getData('text/plain');

              if (commandIdStr) {
                const droppedCommandId = parseInt(commandIdStr, 10);
                appAPI().moveCommandBetween(
                  droppedCommandId,
                  command.id,
                  data.commands[commandIndex + 1]?.id || null,
                );
              }
            }}
          />
        {/if}
      </div>
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
      / 400px;
  }

  .editing {
    grid-template-columns: 400px 1fr;
  }
</style>
