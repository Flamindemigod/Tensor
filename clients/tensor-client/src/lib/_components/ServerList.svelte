<script lang="ts">
  import * as ContextMenu from "$lib/components/ui/context-menu";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import * as Dialog from "$lib/components/ui/dialog";
  import DialogClose from "node_modules/bits-ui/dist/bits/dialog/components/dialog-close.svelte";
  import { SERVERS } from "$context/serverLists";
  import {
    ACTIVE_SERVER,
    setServerActive,
    unsetServerActive,
  } from "$context/activeServer";
  import { setServerIcon } from "$context/serverLists";
  import {
    Bell,
    Download,
    Image,
    Link1,
    LinkNone1,
    Avatar,
  } from "radix-icons-svelte";
  import Input from "$lib/components/ui/input/input.svelte";
  import { onMount } from "svelte";
  import {
    readable,
    writable,
    type Readable,
    type Writable,
  } from "svelte/store";
  import Button from "$lib/components/ui/button/button.svelte";

  function blobToBase64(blob: Blob): Promise<string> {
    return new Promise((resolve, _) => {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result as string);
      reader.readAsDataURL(blob);
    });
  }

  let context: "ServerIcon" | null = null;
  let image_upload: HTMLInputElement;
  let files: Writable<string[]> = writable([] as string[]);
  const file_handler = async () => {
    if (!!image_upload.files) {
      const data = Array.prototype.slice.call(image_upload.files) as File[];

      files.set(
        await Promise.all(data.map(async (d) => await blobToBase64(d))),
      );
    }
  };
  onMount(() => {
    window.addEventListener("paste", (e) => {
      if (!!e.clipboardData?.files) image_upload.files = e.clipboardData.files;
      file_handler();
    });
  });
</script>

<div
  id="serverSidebar"
  class="flex flex-col gap-2 items-start bg-white/5 p-2 overflow-y-auto flex-shrink-0"
>
  {#each $SERVERS as server}
    <Dialog.Root
      onOpenChange={(open) => {
        if (!open) context = null;
      }}
    >
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
            <ContextMenu.Item class="col-span-full grid grid-cols-subgrid">
              <Dialog.Trigger
                on:click={() => {
                  context = "ServerIcon";
                }}
                class="col-span-full grid grid-cols-subgrid justify-start"
              >
                <Image />
                <p class="text-start">Set Server Icon</p>
              </Dialog.Trigger>
            </ContextMenu.Item>
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
      {#if context === "ServerIcon"}
        <Dialog.Content>
          <Dialog.Header>
            <Dialog.Title>Set Custom Server Icon</Dialog.Title>
            <Dialog.Description>
              <Input
                on:change={file_handler}
                bind:ref={image_upload}
                type="file"
                id="img"
                name="img"
                accept="image/*"
              />
              {#each $files as image_url}
                <div
                  class="mt-2 relative overflow-clip font-mono text-3xl rounded-3xl hover:rounded-md transition-all bg-white/10 hover:bg-white/30 w-16 h-16 grid place-items-center uppercase"
                >
                  <img
                    loading="eager"
                    src={image_url}
                    alt={`Server Icon for ${server.server_name}`}
                    class="object-cover w-full h-full"
                  />
                </div>
              {/each}
              {#if !!$files.length}
                <DialogClose>
                  <Button
                    on:click={() => {
                      setServerIcon(server, $files.at(0));
                    }}>Use as New Server Icon</Button
                  >
                </DialogClose>
              {/if}
            </Dialog.Description>
          </Dialog.Header>
        </Dialog.Content>
      {/if}
    </Dialog.Root>
  {/each}
</div>
