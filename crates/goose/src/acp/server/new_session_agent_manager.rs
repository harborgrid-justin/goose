use super::GooseAcpAgent;
use agent_client_protocol::schema::{NewSessionRequest, NewSessionResponse};
use agent_client_protocol::{Client, ConnectionTo};

impl GooseAcpAgent {
    #[allow(dead_code)]
    pub(super) async fn handle_new_session_agent_manager_experiment(
        &self,
        cx: &ConnectionTo<Client>,
        args: NewSessionRequest,
    ) -> Result<NewSessionResponse, agent_client_protocol::Error> {
        self.on_new_session(cx, args).await
    }
}
