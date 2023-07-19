<script lang="ts" context="module">
  const formatByGranularity: [(x: Date) => number, string][] = [
    [(x) => x.getFullYear(), 'y LLL dd HH:mm:ss.SSS'],
    [(x) => x.getMonth(), 'LLL dd HH:mm:ss.SSS'],
    [(x) => x.getDate(), 'LLL dd HH:mm:ss.SSS'],
  ];

  const defaultFormat = 'HH:mm:ss.SSS';
</script>

<script lang="ts">
  import { CommandLineSource, type CommandLogLine } from '$lib/types';
  import { afterUpdate, onMount } from 'svelte';
  import { format } from 'date-fns';
  import ansiToHtml from 'ansi-to-html';

  export let logLines: CommandLogLine[];

  let ansiToHtmlConverter = new ansiToHtml({
    fg: 'inherit',
    bg: 'transparent',
    escapeXML: true,
  });

  let consoleDiv: HTMLDivElement;
  let isCurrentlyScrolledToBottom = true;

  let dateFormatString = defaultFormat;

  $: {
    if (logLines.length < 2) {
      dateFormatString = defaultFormat;
    } else {
      const firstDate = new Date(logLines[0].timestamp);
      const lastDate = new Date(logLines[logLines.length - 1].timestamp);

      dateFormatString = defaultFormat;

      for (const [key, format] of formatByGranularity) {
        if (key(firstDate) !== key(lastDate)) {
          dateFormatString = format;
          break;
        }
      }
    }
  }

  function scrollToBottom() {
    if (!consoleDiv) return;
    consoleDiv.scrollTop = consoleDiv.scrollHeight;
    isCurrentlyScrolledToBottom = true;
  }

  function onScroll() {
    isCurrentlyScrolledToBottom =
      Math.abs(consoleDiv.scrollHeight - consoleDiv.clientHeight - consoleDiv.scrollTop) < 1;
  }

  onMount(() => {
    scrollToBottom();
  });

  afterUpdate(() => {
    if (isCurrentlyScrolledToBottom) scrollToBottom();
  });
</script>

<div
  class="flex-1 bg-zinc-700 mx-4 mb-4 p-4 text-white font-mono overflow-auto text-sm cli-grid"
  bind:this={consoleDiv}
  on:scroll={onScroll}
>
  {#each logLines as logLine (logLine.id)}
    <div class="text-right select-none text-zinc-400">
      {format(new Date(logLine.timestamp), dateFormatString)}
    </div>
    <div class="text-red-800">
      {logLine.source === CommandLineSource.STDERR ? 'E' : ''}
    </div>
    <div
      class="whitespace-pre-wrap min-w-0"
      class:text-zinc-400={logLine.source === CommandLineSource.INFO}
    >
      {@html ansiToHtmlConverter.toHtml(logLine.line)}
    </div>
  {/each}
</div>

<style lang="postcss">
  .cli-grid {
    @apply grid gap-x-2;

    grid-template-columns: auto auto 1fr;
    grid-auto-rows: min-content;
    align-content: safe end;
  }
</style>
