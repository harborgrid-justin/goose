# State Machine — Work in Progress

This module is the WIP unrolled agent loop, replacing the monolithic
`Agent::reply_internal`. It is gated behind the `GOOSE_STATE_MACHINE`
environment variable.

The thesis: **the conversation is the state.** Operations observe the
current `Session` and return declarative outcomes; the machine applies them.
Persistence, event emission, and orchestration live in the machine driver,
not in operations.

---

## Layout

```
state_machine/
├── mod.rs          # public surface: `reply` + `enabled` flag check
├── machine.rs      # the driver: assemble ops, run loop, apply outcomes
├── operation.rs    # Operation trait, Emitter, TurnOutcome
├── ops_llm.rs      # chat LLM operation (streaming)
└── WIP.md          # this file
```

---

## Current status

- [x] `GOOSE_STATE_MACHINE=1` flag dispatch from `Agent::reply`
- [x] `Operation` trait with `applies(&Session)` + `run(&Session, Emitter) -> TurnOutcome`
- [x] Streaming `LlmOperation` (chat-only, no tools)
- [x] Machine driver applies `AppendMessages`, `ReplaceConversation`, `YieldToClient`
- [ ] More operations (see backlog below)
- [ ] `UpdateSession(SessionUpdate)` outcome variant
- [ ] Cancellation token plumbed through the machine

---

## Shape

### `Operation`

```rust
#[async_trait]
pub trait Operation: Send + Sync {
    fn name(&self) -> &'static str;
    fn applies(&self, session: &Session) -> bool;
    async fn run(&self, session: &Session, emit: Emitter) -> Result<TurnOutcome>;
}
```

Ops take `&Session` (read-only — the conversation IS the state) and an
`Emitter` (a sender for `AgentEvent`s the client should see in real time).
They stream 0+ events through the emitter and return one `TurnOutcome`.

Construction-time dependencies (providers, system prompts, extension
managers, per-call knobs like `max_turns`) are passed to the op's
constructor — never on `Session`. The state machine itself does not know
they exist.

### `TurnOutcome`

```rust
pub enum TurnOutcome {
    AppendMessages(Vec<Message>),
    ReplaceConversation(Conversation),
    // TODO: UpdateSession(SessionUpdate)
    YieldToClient,
}
```

The machine commits the outcome to `session` and persists it via
`SessionManager`. It does **not** auto-emit events for `AppendMessages` —
ops already streamed what they wanted visible. The `Conversation` type
handles chunk merging on `push`, so the LLM op can push raw provider chunks
and get a clean merged final result.

### Machine driver

The driver (`machine::reply`) is the only place that:

- persists messages and conversations via `SessionManager`
- mutates `session` (push to conversation, replace, future field updates)
- runs the `applies`/`run` loop
- turns ops' emitted events into the client `AgentEvent` stream
- forwards `HistoryReplaced` on `ReplaceConversation`

Loop termination: either an op returns `YieldToClient`, or no op applies
(every op's `applies` returned false).

---

## Operations to port

Roughly in order of value, with the code in `agents/agent.rs` they replace:

| Operation | Replaces | Notes |
|---|---|---|
| **LLM** | `stream_response_from_provider` + the main `while let Some(next) = stream.next()` arms | Landed (chat-only, streaming). Constructor: `(Arc<dyn Provider>, system_prompt, tools)`. Needs `tools` parameter once tool ops land. |
| **Tool approval** | `tool_inspection_manager.inspect_tools` + `process_inspection_results_with_permission_inspector` + `handle_approval_tool_requests` | Annotates `ToolRequest`s with approval state. YOLO short-circuits. |
| **Tool execution** | `handle_approved_and_denied_tools` + `combined.next()` `tokio::select!` loop + frontend tool sub-flow | Runs approved tools, appends responses. Sub-handles frontend tools and elicitation draining. |
| **Compaction** | `ProviderError::ContextLengthExceeded` arm + `check_if_compaction_needed` block in `reply()` | Triggered when LLM op yields ContextLengthExceeded OR pre-turn token count > threshold. Returns `ReplaceConversation`. |
| **Tool-call pair compaction** | `crate::context_mgmt::maybe_summarize_tool_pairs` background task | Synchronous first cut; revisit backgrounding if it regresses latency. |
| **Elicitation** | `drain_elicitation_messages` + `ActionRequiredManager` calls | When a tool request needs elicitation and has no response: `YieldToClient` (after emitting an elicitation request). Re-entry via `reply()` with `ElicitationResponse`. |
| **Max turns** | `if turns_taken > max_turns` block | Trivial. Counter is per-op or per-machine state (TBD when needed). |
| **Retry / goal / grind / final-output** | `handle_retry_logic` + `goal` / `grind` / `final_output` blocks | One op when last assistant message has no tool requests. May append a nudge or `YieldToClient`. |
| **Subagent sync** | `subagent_handler` + `moim::inject_moim` | When subagents have results to report: append, run another turn. |
| **Hooks (cross-cutting)** | scattered `hook_manager.emit(...)` and `emit_blocking(...)` calls | Run alongside ops, not in the ordered list. `UserPromptSubmit` on entry, `Stop` before `YieldToClient`. Denial flows back via session state. |
| **Slash commands** | `execute_command` block in `reply()` | First-turn-only op. May short-circuit with an assistant response and `YieldToClient`. |
| **Refresh tools after `manage_extensions`** | `tools_updated` block | Either a tail-step of the Tool execution op or a separate op. |

---

## Open questions

- **Where do turn counters live?** Today there are none. When the max-turns
  op lands, it needs to count turns across loop iterations. Options: pass a
  mutable counter into the op constructor (`Arc<AtomicU32>`), or reintroduce
  a thin `TurnState { session, counters }` wrapper. Defer until needed.
- **System prompt rebuild policy.** Baked at construction is fine for chat;
  for tools we need to rebuild when extensions change. Likely an
  `Arc<PromptManager>` on the LLM op and a session-side version marker the
  op reads.
- **Persistence granularity.** Per-outcome (write after each append) — same
  as today's behaviour. Fine.
- **Subagent reporting** is push-driven today (subagent posts back via a
  channel). The state-machine framing wants pull-driven (a `SubagentSync`
  op checks for queued results). Where does the buffer live? Probably on
  the agent, not the session.
- **Hooks as "cross-cutting"** — likely the machine fires hooks at
  well-known points (turn start, before LLM, after tool execution, before
  yield) rather than ops doing it.
- **Cancellation.** `CancellationToken` is checked at many points in the old
  loop. Needs to be a parameter to `reply`, checked between turns by the
  machine, and inside long ops at await points.

---

## Migration steps remaining

1. Add ops in the order in the backlog table.
2. Add `cancel_token: Option<CancellationToken>` to `reply` and plumb it
   through.
3. Fold `reply()` entry-point logic in (elicitation response, slash
   commands, `UserPromptSubmit` hook, pre-turn auto-compact) as
   first-turn-only ops.
4. Tests: `crates/goose/tests/state_machine/` exercising one op at a time
   against a mock provider. Add a CI matrix entry that runs the existing
   reply-stream tests with `GOOSE_STATE_MACHINE=1`.
5. Flip the flag default after a release with no regressions.
6. Delete `reply_internal` and friends.
7. Public API for swapping the pipeline (`AgentConfig::operations`,
   dynamic insert/remove).
