<script lang="ts">
  import { ACTIVE_SERVER } from "$context/activeServer";
  import Button from "$lib/components/ui/button/button.svelte";
  import Textarea from "$lib/components/ui/textarea/textarea.svelte";
  import { Face, PaperPlane } from "radix-icons-svelte";
  import {
    incoming_message_validator,
    type IncomingMessage,
  } from "$types/message";
  import { writable, type Writable } from "svelte/store";
  import { onDestroy, onMount } from "svelte";
  import clsx from "clsx";
  import * as Popover from "$lib/components/ui/popover";
  import { sleep } from "node_modules/bits-ui/dist/internal";
  import "emoji-picker-element";

  let chatLog: HTMLDivElement;
  let text: string;
  let messages: Writable<Map<string, IncomingMessage>> = writable(new Map());

  if (!!$ACTIVE_SERVER) {
    $ACTIVE_SERVER.ws.onmessage = async (d) => {
      const data = incoming_message_validator.parse(JSON.parse(d.data));
      messages.update((m) => m.set(data.message_uuid, data));
      await sleep(100);
      chatLog.scrollTo({
        top: chatLog.scrollHeight - chatLog.clientHeight,
      });
    };
  }

  const text_submit = (e?: SubmitEvent) => {
    if (!!e) e.preventDefault();
    if (!!text) {
      const data = {
        op: 0,
        allowed_mentions: true,
        message_uuid: null,
        message: text,
      };
      $ACTIVE_SERVER?.ws.send(JSON.stringify(data));
      text = "";
    }
  };
  let height: number;
  let width: number;
  const getSize = () => {
    const header = document.querySelector("header");
    const sidebar = document.getElementById("serverSidebar");
    height = document.body.clientHeight - (header?.clientHeight ?? 0);
    width = document.body.clientWidth - (sidebar?.clientWidth ?? 0);
  };
  const handle_emoji = (e: any) => {
    text = (text ?? "") + e.detail.unicode;
  };
  onMount(() => {
    getSize();
    document.addEventListener("resize", getSize);
    document.addEventListener("emoji-click", handle_emoji);
  });
  onDestroy(() => {
    document.removeEventListener("resize", getSize);
    document.removeEventListener("emoji-click", handle_emoji);
  });
</script>

<div class="flex flex-col" style={`height: ${height}px; width:${width}px`}>
  <div
    class="border-b-white border-b-2 bg-amber-300/15 self-start p-2 font-medium w-full"
  >
    <h1 class="text-3xl">{$ACTIVE_SERVER?.server.server_name}</h1>
  </div>
  <div
    bind:this={chatLog}
    id="ChatLog"
    class="overflow-y-auto overflow-x-clip self-stretch rounded-md h-full p-2 border-white border flex flex-col gap-2 m-2"
  >
    {#each $messages.values() as message}
      <div class="flex flex-col gap-2">
        {#if !message.is_server_message}
          <div class="flex gap-2 items-center">
            <div>
              {`<<!${message.author_uuid}>>`}
            </div>
            <div class="text-xs font-medium">
              {new Date(message.unix_time * 1000).toLocaleString(undefined, {
                timeStyle: "short",
                dateStyle: "medium",
              })}
            </div>
          </div>
        {/if}
        <span
          class={clsx(
            "whitespace-pre-wrap break-words",
            message.is_server_message && "text-slate-500 text-center",
          )}>{message.data}</span
        >
      </div>
    {/each}
  </div>
  <div class="self-end w-full p-2 rounded-md">
    <form class="flex gap-2 items-center" name="textinput" on:submit={text_submit}>
      <Textarea
        on:keypress={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            text_submit();
          }
        }}
        bind:value={text}
        form="textinput"
        placeholder="Someone looking cute today"
        class="min-h-0 h-12"
      ></Textarea>
      <Popover.Root>
        <Popover.Trigger class="grid place-items-center h-full">
          <Button variant="outline" class="p-2 aspect-square h-12"><Face /></Button>
        </Popover.Trigger>
        <Popover.Content class="h-96 w-full max-w-96" collisionPadding={2}>
          <emoji-picker class="w-full h-full" />
        </Popover.Content>
      </Popover.Root>
      <Button variant="secondary" class="p-2 aspect-square h-12" type="submit"
        ><PaperPlane /></Button
      >
    </form>
  </div>
</div>
