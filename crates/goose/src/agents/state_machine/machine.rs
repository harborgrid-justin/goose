use std::sync::Arc;

use anyhow::Result;
use async_stream::try_stream;
use futures::stream::BoxStream;
use tokio::sync::mpsc;

use crate::agents::state_machine::operation::{Emitter, Operation, TurnOutcome};
use crate::agents::state_machine::ops_llm::LlmOperation;
use crate::agents::types::SessionConfig;
use crate::agents::{Agent, AgentEvent};
use crate::conversation::message::Message;
use crate::conversation::Conversation;
use crate::session::Session;

/// State-machine replacement for `Agent::reply`.
pub async fn reply(
    agent: &Agent,
    user_message: Message,
    session_config: SessionConfig,
) -> Result<BoxStream<'_, Result<AgentEvent>>> {
    let session_manager = agent.config.session_manager.clone();

    session_manager
        .add_message(&session_config.id, &user_message)
        .await?;

    let mut session = session_manager
        .get_session(&session_config.id, true)
        .await?;

    let working_dir = session.working_dir.clone();
    let (_tools, _toolshim_tools, system_prompt) = agent
        .prepare_tools_and_prompt(&session_config.id, &working_dir)
        .await?;

    let provider = agent.provider().await?;

    // for future use -- now just to match the signature
    let _ = session_config;

    let operations: Vec<Arc<dyn Operation>> =
        vec![Arc::new(LlmOperation::new(provider, system_prompt))];

    Ok(Box::pin(try_stream! {
        loop {
            let Some(op) = operations.iter().find(|op| op.applies(&session)).cloned() else {
                break;
            };
            tracing::debug!(target: "goose::state_machine", op = op.name(), "running operation");

            let (tx, mut rx) = mpsc::channel::<AgentEvent>(32);
            let emitter = Emitter::new(tx);

            let outcome: TurnOutcome = {
                let op_fut = op.run(&session, emitter);
                tokio::pin!(op_fut);
                let result = loop {
                    tokio::select! {
                        biased;
                        Some(event) = rx.recv() => yield event,
                        result = &mut op_fut => break result,
                    }
                };
                result?
            };

            // Op returned; its Emitter dropped; channel closed. Drain leftovers.
            while let Some(event) = rx.recv().await {
                yield event;
            }

            match outcome {
                TurnOutcome::AppendMessages(messages) => {
                    for msg in &messages {
                        session_manager.add_message(&session.id, msg).await?;
                    }
                    let conversation = session
                        .conversation
                        .get_or_insert_with(Conversation::empty);
                    for msg in messages {
                        conversation.push(msg);
                    }
                }
                TurnOutcome::ReplaceConversation(conversation) => {
                    session_manager
                        .replace_conversation(&session.id, &conversation)
                        .await?;
                    session.conversation = Some(conversation.clone());
                    yield AgentEvent::HistoryReplaced(conversation);
                }
                TurnOutcome::YieldToClient => break,
            }
        }
    }))
}
