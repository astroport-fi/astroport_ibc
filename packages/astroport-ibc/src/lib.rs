use std::ops::RangeInclusive;

pub const TIMEOUT_LIMITS: RangeInclusive<u64> = 60..=43200;

/// 2 weeks - 2 months
pub const SIGNAL_OUTAGE_LIMITS: RangeInclusive<u64> = 1209600..=5184000;
