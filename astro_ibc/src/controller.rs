use astroport_governance::assembly::ProposalMessage;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub timeout: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcProposal {
    pub id: u64,
    pub messages: Vec<ProposalMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IbcProposalState {
    InProgress {},
    Succeed {},
    Failed {},
}

impl Display for IbcProposalState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IbcProposalState::InProgress {} => write!(f, "in_progress"),
            IbcProposalState::Succeed {} => write!(f, "succeed"),
            IbcProposalState::Failed {} => write!(f, "failed"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    IbcExecuteProposal {
        channel_id: String,
        proposal_id: u64,
        messages: Vec<ProposalMessage>,
    },
}
