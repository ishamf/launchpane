<script lang="ts">
  import { icons } from '$lib/constants';

  export let icon: keyof typeof icons;
  export let title: string;
</script>

<svg viewBox="0 0 24 24" class="fill-zinc-400">
  <title>{title}</title>
  <use href={icons[icon].join('#')} />
</svg>
