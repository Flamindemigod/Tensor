<script lang="ts">
  import { ACTIVE_SERVER } from "$context/activeServer";
  import Button from "$lib/components/ui/button/button.svelte";
  import Textarea from "$lib/components/ui/textarea/textarea.svelte";
  import { PaperPlane } from "radix-icons-svelte";
  import {
    incoming_message_validator,
    type IncomingMessage,
  } from "$types/message";
  import { writable, type Writable } from "svelte/store";
  import { onDestroy } from "svelte";

  let text: string;
  let messages: Writable<Map<string, IncomingMessage>> = writable(new Map());

  if (!!$ACTIVE_SERVER) {
    $ACTIVE_SERVER.ws.onmessage = (d) => {
      console.log(d.data);
      console.log(JSON.parse(d.data));
      const data = incoming_message_validator.parse(JSON.parse(d.data));
      console.log(data);
      messages.update((m) => m.set(data.message_uuid, data));
    };
  }

  const text_submit = (e?: SubmitEvent) => {
    if (!!e) e.preventDefault();
    const data = {
      op: 0,
      allowed_mentions: true,
      message_uuid: null,
      message: text,
    };
    $ACTIVE_SERVER?.ws.send(JSON.stringify(data));
    text = "";
  };
  let height: number;
  const getHeight = () => {
    const header = document.querySelector("header");
    height = document.body.clientHeight - (header?.clientHeight ?? 0);
  };
  document.addEventListener("resize", getHeight);
  onDestroy(() => {
    document.removeEventListener("resize", getHeight);
  });
</script>

<div class="flex flex-col w-full" style={`height: ${height}px`}>
  <div
    class="border-b-white border-b-2 bg-amber-300/15 self-start p-2 font-medium w-full"
  >
    <h1 class="text-3xl">{$ACTIVE_SERVER?.server.server_name}</h1>
  </div>
  <div
    id="ChatLog"
    class="overflow-y-scroll self-stretch rounded-md h-full p-2 border-white border flex flex-col gap-2 m-2"
  >
    {#each $messages.values() as message}
      <div class="whitespace-pre-line">{message.data}</div>
    {/each}
  </div>
  <div class="self-end w-full p-2 rounded-md">
    <form class="flex gap-2" name="textinput" on:submit={text_submit}>
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
        class="h-16"
      />
      <Button variant="outline" class="p-2 h-16 w-16" type="submit"
        ><PaperPlane /></Button
      >
    </form>
  </div>
</div>
