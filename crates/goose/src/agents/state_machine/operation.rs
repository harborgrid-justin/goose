use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::agents::AgentEvent;
use crate::conversation::message::Message;
use crate::conversation::Conversation;
use crate::session::Session;

/// One step in the agent loop. The first op whose `applies` returns true
/// gets to `run` — it streams events via the emitter and returns an outcome.
#[async_trait]
pub trait Operation: Send + Sync {
    fn name(&self) -> &'static str;

    fn applies(&self, session: &Session) -> bool;

    async fn run(&self, session: &Session, emit: Emitter) -> Result<TurnOutcome>;
}

pub enum TurnOutcome {
    /// Append messages to the conversation
    AppendMessages(Vec<Message>),

    /// Replace the entire conversation (compaction, `/clear`, …).
    ReplaceConversation(Conversation),

    // TODO: `UpdateSession(SessionUpdate)` — variants added as ops need them
    // (provider name, model config, goose_mode, …).

    /// Hand control back to the caller and stop the loop
    YieldToClient,
}


pub struct Emitter {
    tx: mpsc::Sender<AgentEvent>,
}

impl Emitter {
    pub fn new(tx: mpsc::Sender<AgentEvent>) -> Self {
        Self { tx }
    }

    /// Drops silently if the receiver is gone (caller cancelled the stream).
    pub async fn emit(&self, event: AgentEvent) {
        let _ = self.tx.send(event).await;
    }
}

