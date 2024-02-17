<script lang="ts">
  import * as ContextMenu from "$lib/components/ui/context-menu";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { SERVERS } from "$context/serverLists";
  import {
    ACTIVE_SERVER,
    setServerActive,
    unsetServerActive,
  } from "$context/activeServer";
  import {
    Bell,
    Download,
    Image,
    Link1,
    LinkNone1,
    Avatar,
  } from "radix-icons-svelte";
</script>

<div
  class="flex flex-col gap-2 items-start bg-white/5 p-2 overflow-y-auto flex-shrink-0"
>
  {#each $SERVERS as server}
    <ContextMenu.Root>
      <Tooltip.Root>
        <Tooltip.Trigger>
          <ContextMenu.Trigger>
            <button
              class="relative overflow-clip font-mono text-3xl rounded-3xl hover:rounded-md transition-all bg-white/10 hover:bg-white/30 w-16 h-16 grid place-items-center uppercase"
              on:click={() => {
                if (server === $ACTIVE_SERVER?.server) {
                  unsetServerActive();
                } else {
                  setServerActive(server);
                }
              }}
            >
              {#if server.server_icon !== null}
                <img
                  loading="eager"
                  src={server.server_icon}
                  alt={`Server Icon for ${server.server_name}`}
                  class="object-cover w-full h-full"
                />
              {:else}
                {server.server_name.at(0)}
              {/if}
              {#if server === $ACTIVE_SERVER?.server}
                <Link1 />
              {:else}
                <LinkNone1 />
              {/if}
            </button>
          </ContextMenu.Trigger>
        </Tooltip.Trigger>
        <Tooltip.Content side="right">
          <p>{server.server_name}</p>
        </Tooltip.Content>
        <ContextMenu.Content class="grid grid-cols-[32px_1fr]">
          <ContextMenu.Item class="col-span-full grid grid-cols-subgrid"
            ><Image />
            <p>Set Server Icon</p></ContextMenu.Item
          >
          <ContextMenu.Item class="col-span-full grid grid-cols-subgrid"
            ><Avatar />
            <p>Set Display Name</p></ContextMenu.Item
          >
          <ContextMenu.Item class="col-span-full grid grid-cols-subgrid"
            ><Bell />
            <p>Notification Settings</p></ContextMenu.Item
          >
          <ContextMenu.Item class="col-span-full grid grid-cols-subgrid"
            ><Download />
            <p>Export Server</p></ContextMenu.Item
          >
        </ContextMenu.Content>
      </Tooltip.Root>
    </ContextMenu.Root>
  {/each}
</div>
