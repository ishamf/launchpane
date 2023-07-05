<script lang="ts">
  import '../app.css';
  import { WindowState } from '$lib/types';
  import type { PageData } from './$types';
    import { page } from '$app/stores';

  $: windowState = $page.data.windowState;

  $: console.log(windowState);

  $: {
    electronAPI.setWindowState(windowState);
  }
</script>

<div class="w-full h-full flex flex-row">
  <div class="flex flex-col flex-[3]">
    <a class="p-4 m-4 bg-zinc-200 hover:bg-zinc-300" href="/add">Add Command</a>
  </div>

  <slot />
</div>
