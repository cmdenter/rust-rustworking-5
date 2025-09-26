export const idlFactory = ({ IDL }) => {
  const ToolCallArgument = IDL.Record({
    'value' : IDL.Text,
    'name' : IDL.Text,
  });
  const FunctionCall = IDL.Record({
    'name' : IDL.Text,
    'arguments' : IDL.Vec(ToolCallArgument),
  });
  const ToolCall = IDL.Record({ 'id' : IDL.Text, 'function' : FunctionCall });
  const AssistantMessage = IDL.Record({
    'content' : IDL.Opt(IDL.Text),
    'tool_calls' : IDL.Vec(ToolCall),
  });
  const ChatMessage = IDL.Variant({
    'tool' : IDL.Record({ 'content' : IDL.Text, 'tool_call_id' : IDL.Text }),
    'user' : IDL.Record({ 'content' : IDL.Text }),
    'assistant' : AssistantMessage,
    'system' : IDL.Record({ 'content' : IDL.Text }),
  });
  const PoemCycle = IDL.Record({
    'id' : IDL.Nat64,
    'title' : IDL.Text,
    'next_prompt' : IDL.Text,
    'poem' : IDL.Text,
    'cycle_number' : IDL.Nat64,
    'created_at' : IDL.Nat64,
    'bukowski_style_score' : IDL.Opt(IDL.Float32),
  });
  const Result = IDL.Variant({ 'Ok' : PoemCycle, 'Err' : IDL.Text });
  const StoredMessage = IDL.Record({
    'content' : IDL.Text,
    'role' : IDL.Text,
    'timestamp' : IDL.Nat64,
  });
  const Conversation = IDL.Record({
    'id' : IDL.Nat64,
    'title' : IDL.Text,
    'updated_at' : IDL.Nat64,
    'created_at' : IDL.Nat64,
    'message_count' : IDL.Nat64,
  });
  const ConversationWithMessages = IDL.Record({
    'messages' : IDL.Vec(StoredMessage),
    'conversation' : Conversation,
  });
  const PoetState = IDL.Record({
    'total_poems' : IDL.Nat64,
    'last_updated' : IDL.Nat64,
    'genesis_prompt' : IDL.Text,
    'current_cycle' : IDL.Nat64,
  });
  return IDL.Service({
    'chat' : IDL.Func([IDL.Vec(ChatMessage)], [IDL.Text], []),
    'chat_with_storage' : IDL.Func(
        [IDL.Opt(IDL.Nat64), IDL.Vec(ChatMessage)],
        [IDL.Nat64, IDL.Text],
        [],
      ),
    'delete_conversation' : IDL.Func([IDL.Nat64], [IDL.Bool], []),
    'evolve_poet' : IDL.Func([], [Result], []),
    'get_all_poems' : IDL.Func([], [IDL.Vec(PoemCycle)], ['query']),
    'get_conversation_messages' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(StoredMessage)],
        ['query'],
      ),
    'get_conversation_with_messages' : IDL.Func(
        [IDL.Nat64],
        [IDL.Opt(ConversationWithMessages)],
        ['query'],
      ),
    'get_conversations' : IDL.Func([], [IDL.Vec(Conversation)], ['query']),
    'get_current_poem' : IDL.Func([], [IDL.Opt(PoemCycle)], ['query']),
    'get_poem_by_cycle' : IDL.Func(
        [IDL.Nat64],
        [IDL.Opt(PoemCycle)],
        ['query'],
      ),
    'get_poet_state' : IDL.Func([], [IDL.Opt(PoetState)], ['query']),
    'prompt' : IDL.Func([IDL.Text], [IDL.Text], []),
    'reset_poet' : IDL.Func([], [IDL.Bool], []),
    'update_conversation_title' : IDL.Func(
        [IDL.Nat64, IDL.Text],
        [IDL.Bool],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
