// Tensor Message Protocol
// C Style Header to Outline Protocol Implementions (Not to be used for prod but as refrence instead)

//MIT License
//
// Copyright (c) 2024 Flamindemigod
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

typedef struct {
  char message_uuid[16];
  char author_uuid[16];
  char *data;
  bool edited;
  bool is_mentioned;
  bool is_server_message;
  int unix_time; //Time in Seconds from from UNIX_EPOCH
} Message;

typedef enum {
  NEW_MESSAGE,
  EDIT_MESSAGE,
  DELETE_MESSAGE,
} MessageOps;


typedef struct{
  MessageOps op;
  bool allowed_mentions;
  char message_uuid[16];
  char *message; //If message mentions a user, Message Must Embed the user uuid within <<!uuid>>
} ClientSend;

// Client 
Message on_message(void *data);
void    send(ClientSend c);
