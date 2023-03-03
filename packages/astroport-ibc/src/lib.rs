use std::{
    fmt::{Display, Formatter, Result},
    ops::RangeInclusive,
};

use cosmwasm_schema::cw_serde;

pub const TIMEOUT_LIMITS: RangeInclusive<u64> = 60..=600;

// TODO: replace all uses of this enum at the end of the merge
#[cw_serde]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    InProgress,
    Failed,
    Executed,
    Expired,
}
impl Display for ProposalStatus {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            ProposalStatus::Active {} => fmt.write_str("active"),
            ProposalStatus::Passed {} => fmt.write_str("passed"),
            ProposalStatus::Rejected {} => fmt.write_str("rejected"),
            ProposalStatus::InProgress => fmt.write_str("in_progress"),
            ProposalStatus::Failed => fmt.write_str("failed"),
            ProposalStatus::Executed {} => fmt.write_str("executed"),
            ProposalStatus::Expired {} => fmt.write_str("expired"),
        }
    }
}

// TODO: replace all uses of this enum at the end of the merge
#[cw_serde]
pub enum AssemblyExecuteMsg {
    IBCProposalCompleted {
        proposal_id: u64,
        status: ProposalStatus,
    },
}
