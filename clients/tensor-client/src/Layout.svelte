<script lang="ts">
    import Header from "$lib/_components/Header.svelte";
 import { ModeWatcher } from "mode-watcher";
</script>

<ModeWatcher />
<Header /> 
<main class="flex items-center justify-center w-full flex-grow">
  <slot />
</main>
<footer class="" />
