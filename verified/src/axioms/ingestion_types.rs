use vstd::prelude::*;

verus! {

// ─── PublicationRecord ───────────────────────────────────────────────────────

/// Abstract stub for `erdos_graph::db::ingestion::PublicationRecord`.
///
/// Strings (id, title, authors, venue, source) are opaque in Verus, so we
/// model the string-valued fields via uninterpreted spec functions. Only
/// `year` (u32) is kept as a concrete field since it carries verifiable
/// numeric invariants.
pub struct PublicationRecord {
    pub year: u32,
}

/// Opaque predicate: the `source` field is one of "arxiv", "dblp", "zbmath".
///
/// We cannot unfold this definition (it is uninterpreted), so proofs must
/// be given this fact via external_body ensures or require it as a precondition.
pub uninterp spec fn valid_source(r: &PublicationRecord) -> bool;

/// A publication is well-formed when its year is non-zero and its source is
/// one of the three recognised values.
pub open spec fn valid_publication(r: &PublicationRecord) -> bool {
    r.year > 0u32 && valid_source(r)
}

/// External_body constructor: creates a record known to be valid.
///
/// This is the trust boundary – we assert the postcondition and rely on the
/// real implementation to uphold it.
#[verifier::external_body]
pub fn mk_valid_publication(year: u32) -> (r: PublicationRecord)
    requires year > 0u32
    ensures
        r.year == year,
        valid_publication(&r),
{
    unimplemented!()
}

// ─── IngestionConfig ─────────────────────────────────────────────────────────

/// Abstract stub for the subset of `erdos_graph::config::IngestionConfig`
/// that is consumed by the ingestion functions under verification.
///
/// Only the fields that appear in `calculate_date_range` and
/// `chunk_date_range` are modelled here.
pub struct IngestionConfig {
    /// Size of each processing chunk in days.
    pub chunk_size_days: u64,
    /// Number of days to look back in "weekly" mode.
    pub weekly_days: u64,
}

/// Default ingestion config mirrors `IngestionConfig::default()` in the real crate.
#[verifier::external_body]
pub fn default_ingestion_config() -> (c: IngestionConfig)
    ensures
        c.chunk_size_days == 1,
        c.weekly_days == 7,
{
    unimplemented!()
}

}
