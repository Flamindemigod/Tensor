import { writable, type Writable } from "svelte/store";
import { type Server } from "src/types/clientValidator";

export const ACTIVE_SERVER = writable<{ server: Server; ws: WebSocket } | null>(
  null,
);

export const setServerActive = (server: Server) => {
  ACTIVE_SERVER.update((a) => {
    if (!!a?.ws) {
      a.ws.close();
    }
    return {
      server: server,
      ws: new WebSocket(`ws://${server.server_ip}:${server.server_port}`, [
        "Authorization",
        server.client_token,
      ]),
    };
  });
};

export const unsetServerActive = () => {
  ACTIVE_SERVER.update((a) => {
    if (!!a?.ws) {
      a.ws.close();
    }
    return null;
  });
};
