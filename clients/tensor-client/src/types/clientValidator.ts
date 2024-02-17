import { z } from "zod";

export const client_config_validator = z.object({
  server_ip: z.string().ip(),
  server_port: z.number().int().positive().finite().lte(65534),
  server_name: z.string(),
  client_token: z.string().length(16),
});

export enum ServerConnectionStatus {
  Connecting,
  Open,
  Closing,
  Closed,
}

export const server_validator = client_config_validator.extend({
  server_icon: z.string().nullable(),
  // auto_join: z.boolean(),
  // connected: z.boolean(),
  connection_status: z.nativeEnum(ServerConnectionStatus),
});

export type ServerConfig = z.infer<typeof client_config_validator>;
export type Server = z.infer<typeof server_validator>;
