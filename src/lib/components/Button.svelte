<script lang="ts">
  import type { icons } from '$lib/constants';
  import Icon from './Icon.svelte';

  export let icon: keyof typeof icons;
  export let title: string;
  export let href: string | undefined = undefined;

  export let small = false;
  export let altColor = false;

  $: isLink = typeof href === 'string';

  $: props = isLink ? { href } : {};

  $: Component = isLink ? 'a' : 'button';
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- it's a button which already have all the a11y stuff -->
<svelte:element
  this={Component}
  class="block rounded-md w-12 h-12 p-2 border-zinc-300 border cursor-default
        bg-zinc-50 hover:bg-zinc-100 active:bg-zinc-300 transition-colors"
  class:small
  class:altColor
  on:click
  {...props}
>
  <Icon {icon} {title} />
</svelte:element>

<style lang="postcss">
  .small {
    @apply w-8 h-8 p-1;
  }

  .altColor {
    @apply bg-white hover:bg-zinc-100 active:bg-zinc-200;
  }
</style>
