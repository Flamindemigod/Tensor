import { derived, writable } from "svelte/store";
import {
  server_validator,
  type Server,
  type ServerConfig,
} from "src/types/clientValidator";
import { z } from "zod";

export const SERVERS_Internal = writable<Set<Server>>(new Set(), (set) => {
  const data: Server[] = z
    .array(server_validator)
    .catch([])
    .parse(JSON.parse(localStorage.getItem("ServersList")!));
  console.log(data);
  set(
    new Set(
      data
        .map((d) => {
          const k = server_validator.parse(d);
          k.connection_status = 0;
          return k;
        })
        .filter(Boolean),
    ),
  );
  return () => console.log("All Consumers are destroyed");
});

export function addServer(server: ServerConfig) {
  const default_server = {
    connection_status: 0,
    server_icon: null,
  } as Omit<Server, keyof ServerConfig & keyof Server>;
  SERVERS_Internal.update((s) =>
    s.add({ ...default_server, ...server } as Server),
  );
}


export function setServerIcon(server:Server, img: string|undefined){
  if (!img) return; 
  const data = {...server, server_icon: img} 
  console.log("setting server icon ", img);
  SERVERS_Internal.update((s) =>
  new Set([...s.values()].map(si => si === server ? data : si))
    )
}


export const SERVERS = derived(SERVERS_Internal, ($a) => [...$a.values()]);

SERVERS.subscribe((value) => {
  localStorage.setItem("ServersList", JSON.stringify([...value.values()]));
});
