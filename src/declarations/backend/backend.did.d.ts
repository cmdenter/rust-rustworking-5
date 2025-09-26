import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface AssistantMessage {
  'content' : [] | [string],
  'tool_calls' : Array<ToolCall>,
}
export type ChatMessage = {
    'tool' : { 'content' : string, 'tool_call_id' : string }
  } |
  { 'user' : { 'content' : string } } |
  { 'assistant' : AssistantMessage } |
  { 'system' : { 'content' : string } };
export interface Conversation {
  'id' : bigint,
  'title' : string,
  'updated_at' : bigint,
  'created_at' : bigint,
  'message_count' : bigint,
}
export interface ConversationWithMessages {
  'messages' : Array<StoredMessage>,
  'conversation' : Conversation,
}
export interface FunctionCall {
  'name' : string,
  'arguments' : Array<ToolCallArgument>,
}
export interface PoemCycle {
  'id' : bigint,
  'title' : string,
  'next_prompt' : string,
  'poem' : string,
  'cycle_number' : bigint,
  'created_at' : bigint,
  'bukowski_style_score' : [] | [number],
}
export interface PoetState {
  'total_poems' : bigint,
  'last_updated' : bigint,
  'genesis_prompt' : string,
  'current_cycle' : bigint,
}
export type Result = { 'Ok' : PoemCycle } |
  { 'Err' : string };
export interface StoredMessage {
  'content' : string,
  'role' : string,
  'timestamp' : bigint,
}
export interface ToolCall { 'id' : string, 'function' : FunctionCall }
export interface ToolCallArgument { 'value' : string, 'name' : string }
export interface _SERVICE {
  'chat' : ActorMethod<[Array<ChatMessage>], string>,
  'chat_with_storage' : ActorMethod<
    [[] | [bigint], Array<ChatMessage>],
    [bigint, string]
  >,
  'delete_conversation' : ActorMethod<[bigint], boolean>,
  'evolve_poet' : ActorMethod<[], Result>,
  'get_all_poems' : ActorMethod<[], Array<PoemCycle>>,
  'get_conversation_messages' : ActorMethod<[bigint], Array<StoredMessage>>,
  'get_conversation_with_messages' : ActorMethod<
    [bigint],
    [] | [ConversationWithMessages]
  >,
  'get_conversations' : ActorMethod<[], Array<Conversation>>,
  'get_current_poem' : ActorMethod<[], [] | [PoemCycle]>,
  'get_poem_by_cycle' : ActorMethod<[bigint], [] | [PoemCycle]>,
  'get_poet_state' : ActorMethod<[], [] | [PoetState]>,
  'prompt' : ActorMethod<[string], string>,
  'reset_poet' : ActorMethod<[], boolean>,
  'update_conversation_title' : ActorMethod<[bigint, string], boolean>,
}
