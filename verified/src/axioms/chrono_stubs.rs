use vstd::prelude::*;

verus! {

/// Abstract representation of chrono::DateTime<Utc>.
///
/// The `ts` field is a u64 Unix timestamp (seconds since epoch). This lets
/// Verus reason about ordering and arithmetic without depending on the real
/// chrono crate.
#[derive(Clone, Copy)]
pub struct DateTime {
    pub ts: u64,
}

/// Abstract representation of chrono::Duration (days granularity).
#[derive(Clone, Copy)]
pub struct ChronoDuration {
    pub days: u64,
}

// ─── ordering ───────────────────────────────────────────────────────────────

pub open spec fn dt_lt(a: DateTime, b: DateTime) -> bool {
    a.ts < b.ts
}

pub open spec fn dt_le(a: DateTime, b: DateTime) -> bool {
    a.ts <= b.ts
}

// ─── arithmetic ─────────────────────────────────────────────────────────────

/// Adding `days` days to a datetime shifts the timestamp by days * 86400 s.
///
/// Requires that the result does not overflow u64.
pub open spec fn dt_add(dt: DateTime, d: ChronoDuration) -> DateTime
    recommends dt.ts + d.days * 86400 <= u64::MAX
{
    DateTime { ts: (dt.ts + d.days * 86400) as u64 }
}

/// Minimum of two datetimes (mirrors std::cmp::min).
pub open spec fn dt_min(a: DateTime, b: DateTime) -> DateTime {
    if a.ts <= b.ts { a } else { b }
}

// ─── basic lemmas (provable from definitions) ────────────────────────────────

pub proof fn lemma_dt_lt_irrefl(a: DateTime)
    ensures !dt_lt(a, a)
{}

pub proof fn lemma_dt_lt_trans(a: DateTime, b: DateTime, c: DateTime)
    requires dt_lt(a, b), dt_lt(b, c)
    ensures  dt_lt(a, c)
{}

pub proof fn lemma_dt_le_refl(a: DateTime)
    ensures dt_le(a, a)
{}

pub proof fn lemma_dt_min_le_left(a: DateTime, b: DateTime)
    ensures dt_le(dt_min(a, b), a)
{}

pub proof fn lemma_dt_min_le_right(a: DateTime, b: DateTime)
    ensures dt_le(dt_min(a, b), b)
{}

pub proof fn lemma_dt_min_is_one_of(a: DateTime, b: DateTime)
    ensures dt_min(a, b) == a || dt_min(a, b) == b
{}

/// Adding a positive number of days strictly increases the datetime.
pub proof fn lemma_dt_add_positive(dt: DateTime, d: ChronoDuration)
    requires
        d.days > 0,
        dt.ts + d.days * 86400 <= u64::MAX,
    ensures dt_lt(dt, dt_add(dt, d))
{}

// ─── exec constructors (external_body – connect to real chrono at trust boundary) ──

/// Create an abstract DateTime from a raw Unix timestamp.
///
/// Marked external_body because the real implementation would call
/// `DateTime::from_timestamp(ts, 0).unwrap()` on the chrono side.
#[verifier::external_body]
pub fn datetime_from_ts(ts: u64) -> (d: DateTime)
    ensures d.ts == ts
{
    DateTime { ts }
}

/// Create an abstract ChronoDuration from a number of days.
#[verifier::external_body]
pub fn duration_from_days(days: u64) -> (d: ChronoDuration)
    ensures d.days == days
{
    ChronoDuration { days }
}

}
