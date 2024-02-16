<script lang="ts">
    import Button from "$lib/components/ui/button/button.svelte";
    import Input from "$lib/components/ui/input/input.svelte";
    import Label from "node_modules/bits-ui/dist/bits/label/components/label.svelte";
    import { Upload } from "radix-icons-svelte";
    import { addServer } from "src/contexts/serverLists";
    import { client_config_validator } from "src/types/clientValidator";


    const on_file_upload = (e: InputEvent)=> {
      const input = e.target as HTMLInputElement;
      for (const file of input.files!){
        const fileReader = new FileReader();
        fileReader.onload = event => {
          const data = event.target?.result;
          if (!!data){
            const parsed_data = client_config_validator.parse(JSON.parse(data as string))  
            console.log(parsed_data);
            addServer(parsed_data);
            return; 
          }
            console.error("Data was empty");
          
        } // desired file content
        fileReader.onerror = error => console.error(error)
        fileReader.readAsText(file)
      }    
      console.log("File Picker", input.files)
    }
</script>

<header class="relative p-2 isolate bg-amber-400/10 backdrop-blur-lg flex justify-between items-center">
  <a href="/" class="p-2 bg-amber-500/20 rounded-md">
    <img src="./vite.svg" alt="Logo">
  </a>
    <Button class="p-2 flex flex-col gap-1 rounded-md relative">
      <Label for="ConfigUpload" class="flex gap-2 items-center"><Upload /> Upload a new Config</Label>
      <Input on:input={on_file_upload} id="ConfigUpload" type="file" class="absolute inset-0 opacity-0" accept=".conf, application/json" multiple />
    </Button>

</header>
