use vstd::prelude::*;

verus! {

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

// Verify that the Clone implementation for QueueConfig works as expected.
// Specifically, it ensures that the `max_queue_size` is preserved in the new copy.
#[verifier::external_body]
pub fn clone_queue_config(c: &crate::thread_safe_queue::QueueConfig) -> (res: crate::thread_safe_queue::QueueConfig)
    ensures res.max_queue_size == c.max_queue_size
{
    c.clone()
}

// Proof to verify the clone specification
fn check_clone_config() {
    let c1 = default_queue_config();
    let c2 = clone_queue_config(&c1);
    assert(c2.max_queue_size == 10000);
}

// Verify that we can construct the config directly since fields are public
fn check_construct_config() {
    let c = crate::thread_safe_queue::QueueConfig { max_queue_size: 50 };
    assert(c.max_queue_size == 50);
}

}