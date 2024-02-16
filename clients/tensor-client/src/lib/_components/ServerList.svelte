<script lang="ts">
  import {SERVERS} from "$context/serverLists";
  import {setServerActive} from "$context/activeServer";
</script>

<div class="flex flex-col gap-2 items-start h-full self-start p-2 bg-white/5">
  {#each $SERVERS as server}
    <button class="font-mono text-4xl p-4 rounded-3xl hover:rounded-md transition-all bg-white/10 hover:bg-white/30 w-16 h-16 grid place-items-center uppercase" on:click={()=>{setServerActive(server)}}>
      {#if (server.server_icon !== null)} 
        <img src={server.server_icon} alt={`Server Icon for ${server.server_name}`} class="w-full h-full object-center object-cover"/>
      {:else}
        {server.server_name.at(0)}
      {/if}
    </button>
  {/each}
</div>
