use vstd::prelude::*;

// Include the source file directly from the submodule
#[path = "../../erdos-graph/src/utilities/thread_safe_queue.rs"]
mod thread_safe_queue;

verus! {

proof fn test_proof() {
    assert(true);
}

#[verifier::external_type_specification]
pub struct ExQueueConfig(crate::thread_safe_queue::QueueConfig);

#[verifier::external_body]
pub fn default_queue_config() -> (c: crate::thread_safe_queue::QueueConfig)
    ensures c.max_queue_size == 10000
{
    crate::thread_safe_queue::QueueConfig::default()
}

// Check the property of the config
fn check_queue_config() {
    let config = default_queue_config();
    assert(config.max_queue_size == 10000);
}

}
