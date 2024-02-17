import { z } from "zod";

enum MessageOps {
  NEW_MESSAGE,
  EDIT_MESSAGE,
  DELETE_MESSAGE,
}

// typedef struct {
//   char message_uuid[16];
//   char author_uuid[16];
//   char *data;
//   bool edited;
//   bool is_mentioned;
// } Message;

export const incoming_message_validator = z.object({
  message_uuid: z.string().length(16),
  author_uuid: z.string().length(16),
  data: z.string(),
  edited: z.boolean(),
  is_mentioned: z.boolean(),
});

export type IncomingMessage = z.infer<typeof incoming_message_validator>;

// typedef struct{
//   MessageOps op;
//   bool allowed_mentions;
//   char message_uuid[16];
//   char *message; //If message mentions a user, Message Must Embed the user uuid within <<!uuid>>
// } ClientSend;

export const outgoing_message_validator = z.object({
  MessageOps: z.nativeEnum(MessageOps),
  allowed_mentions: z.boolean(),
  message_uuid: z.string().length(16).nullable(),
  message: z.string(),
});

export type OutgoingMessage = z.infer<typeof outgoing_message_validator>;

export const new_message = (message: string, allowed_mentions: boolean) => {
  return {
    MessageOps: MessageOps.NEW_MESSAGE,
    message,
    message_uuid: null,
    allowed_mentions,
  } as OutgoingMessage;
};
