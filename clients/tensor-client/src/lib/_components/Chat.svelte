<script lang="ts">
  import { ACTIVE_SERVER } from "$context/activeServer";
    import Button from "$lib/components/ui/button/button.svelte";
    import Textarea from "$lib/components/ui/textarea/textarea.svelte";
    import { PaperPlane } from "radix-icons-svelte";
  let text:string;
  // const bind_ws = ()=>{
    ACTIVE_SERVER.subscribe(s => {
    if (!s) return null;  
    s.ws.onmessage = (d) => {console.log(d.data)}
    })
  // }
</script>
<div class="flex flex-col self-stretch w-full">
  <div class="border-b-white border-b-2 bg-amber-300/15 self-start w-full m-2 p-2 font-medium">
  <h1 class="text-3xl">{$ACTIVE_SERVER?.server.server_name}</h1>
  </div>
  <div id="ChatLog" class="self-stretch h-full"></div>
  <div class="self-end w-full p-2 rounded-md">
    <form class="flex gap-2" on:submit={(e)=>{
      e.preventDefault()
      const data = {
            op: 0,
            allowed_mentions: true,
            message_uuid: null,
            message: text,
          };
      $ACTIVE_SERVER?.ws.send(JSON.stringify(data));
      text="";
      }}>
    <Textarea bind:value={text} placeholder="Someone looking cute today" class="h-16"/>
    <Button variant="outline" class="p-2 h-16 w-16" type="submit"><PaperPlane  /></Button> 
  </form>
  </div>
</div>
