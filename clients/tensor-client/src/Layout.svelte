<script lang="ts">
  import Header from "$lib/_components/Header.svelte";
  import { ModeWatcher } from "mode-watcher";
</script>

<ModeWatcher />
<Header />
<main class="flex self-stretch flex-grow items-stretch">
  <slot />
</main>
<footer class="self-end" />
