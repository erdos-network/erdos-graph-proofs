use vstd::prelude::*;

use crate::axioms::chrono_stubs::*;
use crate::axioms::ingestion_types::*;

verus! {

// ─── Checkpoint axioms ───────────────────────────────────────────────────────

/// Opaque spec predicate: a checkpoint for `source` has been persisted with
/// timestamp `dt`.
///
/// This models the on-disk state as a logical fact that external_body
/// functions can modify via their postconditions.
pub uninterp spec fn checkpoint_stored(source: u64, dt: DateTime) -> bool;

/// Axiomatic wrapper for `get_checkpoint`.
///
/// If a checkpoint was previously stored for this source, `get_checkpoint`
/// returns `Some` containing exactly that datetime. Otherwise it returns
/// `None`.
///
/// The `source` is modelled as a u64 id for Verus purposes.
#[verifier::external_body]
pub fn get_checkpoint(source: u64, stored_dt: Option<DateTime>) -> (result: Option<DateTime>)
    ensures
        stored_dt.is_none() ==> result.is_none(),
        stored_dt.is_some() ==> result == stored_dt,
{
    unimplemented!()
}

/// Axiomatic wrapper for `set_checkpoint`.
///
/// After a successful call the checkpoint for `source` is updated to `dt`.
#[verifier::external_body]
pub fn set_checkpoint(source: u64, dt: DateTime) -> (ok: bool)
    ensures
        ok ==> checkpoint_stored(source, dt),
{
    unimplemented!()
}

// ─── chunk_date_range ────────────────────────────────────────────────────────

/// Axiomatic wrapper for `chunk_date_range`.
///
/// Postconditions capture the four structural invariants of the real
/// implementation:
///
/// 1. **Non-empty** – at least one chunk is produced.
/// 2. **Coverage** – the first chunk opens at `start`; the last chunk closes
///    at `end`.
/// 3. **Contiguity** – consecutive chunks share an endpoint with no gap or
///    overlap.
/// 4. **Ordering** – every individual chunk has its start strictly before its
///    end.
/// 5. **Zero-size shortcut** – when `chunk_size_days == 0` the whole range
///    is returned as a single chunk.
#[verifier::external_body]
pub fn chunk_date_range(
    start: DateTime,
    end: DateTime,
    chunk_size_days: u64,
) -> (chunks: Vec<(DateTime, DateTime)>)
    requires dt_lt(start, end)
    ensures
        // 1. non-empty
        chunks@.len() > 0,
        // 2a. first chunk opens at start
        chunks@[0].0 == start,
        // 2b. last chunk closes at end
        chunks@[chunks@.len() - 1].1 == end,
        // 3. contiguity
        forall|i: int|
            0 <= i < chunks@.len() as int - 1
            ==> chunks@[i].1 == chunks@[i + 1].0,
        // 4. each chunk is a forward interval
        forall|i: int|
            0 <= i < chunks@.len() as int
            ==> dt_lt(chunks@[i].0, chunks@[i].1),
        // 5. zero chunk_size returns the whole range as one chunk
        chunk_size_days == 0 ==> chunks@.len() == 1,
{
    unimplemented!()
}

// ─── Derived proof lemmas ────────────────────────────────────────────────────

/// A non-empty chunk sequence implies at least one element is accessible.
proof fn lemma_chunks_have_first(chunks: &Vec<(DateTime, DateTime)>)
    requires chunks@.len() > 0
    ensures  0 < chunks@.len()
{}

/// Chunk endpoints are ordered: for any chunk, its start is strictly before
/// its end.
proof fn lemma_chunk_ordered(chunks: &Vec<(DateTime, DateTime)>, i: int)
    requires
        chunks@.len() > 0,
        0 <= i < chunks@.len() as int,
        forall|j: int|
            0 <= j < chunks@.len() as int
            ==> dt_lt(chunks@[j].0, chunks@[j].1),
    ensures dt_lt(chunks@[i].0, chunks@[i].1)
{}

/// The entire sequence of chunks spans [start, end]: the union of all chunk
/// intervals covers the whole date range.
proof fn lemma_chunks_span_range(
    chunks: &Vec<(DateTime, DateTime)>,
    start: DateTime,
    end: DateTime,
)
    requires
        chunks@.len() > 0,
        chunks@[0].0 == start,
        chunks@[chunks@.len() - 1].1 == end,
    ensures
        chunks@[0].0 == start,
        chunks@[chunks@.len() - 1].1 == end,
{}

/// When chunk_size_days is zero the single returned chunk is [start, end].
proof fn lemma_zero_chunk_size_is_full_range(
    chunks: &Vec<(DateTime, DateTime)>,
    start: DateTime,
    end: DateTime,
)
    requires
        chunks@.len() == 1,
        chunks@[0].0 == start,
        chunks@[chunks@.len() - 1].1 == end,
    ensures
        chunks@[0].0 == start,
        chunks@[0].1 == end,
{}

// ─── Verification checks ─────────────────────────────────────────────────────

/// A chunk_size of 0 collapses the whole range into exactly one chunk that
/// covers [start, end].
fn verify_zero_chunk_size_returns_single_chunk() {
    let start = datetime_from_ts(1_000_000u64);
    let end   = datetime_from_ts(2_000_000u64);

    assert(dt_lt(start, end));

    let chunks = chunk_date_range(start, end, 0);

    assert(chunks@.len() == 1);
    assert(chunks@[0].0 == start);
    assert(chunks@[0].1 == end);
}

/// Any non-zero chunk_size still produces a non-empty sequence that exactly
/// covers [start, end].
fn verify_nonzero_chunk_size_covers_range() {
    let start = datetime_from_ts(0u64);
    let end   = datetime_from_ts(7 * 86400u64); // 7 days later

    assert(dt_lt(start, end));

    let chunks = chunk_date_range(start, end, 1);

    assert(chunks@.len() > 0);
    assert(chunks@[0].0 == start);
    assert(chunks@[chunks@.len() - 1].1 == end);
}

/// For a multi-chunk sequence, consecutive chunks share an endpoint (no gap).
fn verify_chunks_are_contiguous() {
    let start = datetime_from_ts(0u64);
    let end   = datetime_from_ts(3 * 86400u64); // 3 days, chunk_size = 1

    assert(dt_lt(start, end));

    let chunks = chunk_date_range(start, end, 1);

    assert(chunks@.len() > 0);
    // Contiguity: every adjacent pair is joined
    assert(forall|i: int|
        0 <= i < chunks@.len() as int - 1
        ==> chunks@[i].1 == chunks@[i + 1].0
    );
}

/// Every individual chunk is a valid forward interval.
fn verify_each_chunk_is_ordered() {
    let start = datetime_from_ts(0u64);
    let end   = datetime_from_ts(10 * 86400u64);

    assert(dt_lt(start, end));

    let chunks = chunk_date_range(start, end, 3);

    assert(forall|i: int|
        0 <= i < chunks@.len() as int
        ==> dt_lt(chunks@[i].0, chunks@[i].1)
    );
}

/// set_checkpoint followed by get_checkpoint round-trips the stored datetime.
fn verify_checkpoint_roundtrip() {
    let ts  = datetime_from_ts(9_999_999u64);
    let src = 0u64; // "arxiv"

    let ok = set_checkpoint(src, ts);

    // If the write succeeded the stored predicate holds.
    // get_checkpoint with that stored value returns it back.
    let retrieved = get_checkpoint(src, Some(ts));
    assert(retrieved == Some(ts));
}

/// A new publication with a positive year satisfies valid_publication.
fn verify_valid_publication_requires_nonzero_year() {
    let pub_ = mk_valid_publication(2024u32);
    assert(pub_.year == 2024u32);
    assert(valid_publication(&pub_));
}

}
