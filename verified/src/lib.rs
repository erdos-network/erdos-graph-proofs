use vstd::prelude::*;

// Include the source file directly from the submodule
#[path = "../../erdos-graph/src/utilities/thread_safe_queue.rs"]
pub mod thread_safe_queue;

pub mod thread_safe_queue_specs;
pub mod axioms;
pub mod ingestion_specs;

verus! {

proof fn test_proof() {
    assert(true);
}

}
