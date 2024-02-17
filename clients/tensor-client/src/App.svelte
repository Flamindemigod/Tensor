<script lang="ts">
  import { ACTIVE_SERVER } from "$context/activeServer";
  import Chat from "$lib/_components/Chat.svelte";
  import ServerList from "$lib/_components/ServerList.svelte";
  import Layout from "./Layout.svelte";
  import { SERVERS } from "$context/serverLists";
</script>

<Layout>
  {#if !$SERVERS.length}
    <div class="grid place-items-center w-full h-full">
      <h1 class="text-4xl">Welcome to Tensor</h1>
      <p>
        Get a client config file from a friend or head <a
          href="https://github.com/Flamindemigod/Tensor"
          class="text-amber-500 bg-amber-300/5 rounded-md hover:underline underline-offset-2 py-0.5 px-1"
          >here</a
        > to host your own server
      </p>
    </div>
  {:else if !$ACTIVE_SERVER}
    <ServerList />
    <div class="grid place-items-center w-full">
      <h1 class="text-4xl">Welcome to Tensor</h1>
      <p>Connect to a server on your server list</p>
    </div>
  {:else}
    <ServerList />
    <Chat />
  {/if}
</Layout>

<style>
</style>
