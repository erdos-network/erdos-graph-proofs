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

// Check default config property
fn check_queue_config() {
    let config = default_queue_config();
    assert(config.max_queue_size == 10000);
}

// Clone preserves max_queue_size
#[verifier::external_body]
pub fn clone_queue_config(c: &crate::thread_safe_queue::QueueConfig) -> (res: crate::thread_safe_queue::QueueConfig)
    ensures res.max_queue_size == c.max_queue_size
{
    c.clone()
}

// Check clone specification
fn check_clone_config() {
    let c1 = default_queue_config();
    let c2 = clone_queue_config(&c1);
    assert(c2.max_queue_size == 10000);
}

// Check direct construction with public fields
fn check_construct_config() {
    let c = crate::thread_safe_queue::QueueConfig { max_queue_size: 50 };
    assert(c.max_queue_size == 50);
}

// External type spec for ThreadSafeQueue (opaque due to private fields)
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
pub struct ExThreadSafeQueue<T>(crate::thread_safe_queue::ThreadSafeQueue<T>);

// Creates new queue with zero active producers
#[verifier::external_body]
pub fn new_queue<T>(config: crate::thread_safe_queue::QueueConfig) -> (q: crate::thread_safe_queue::ThreadSafeQueue<T>)
    ensures
        active_producer_count(&q) == 0,
        !producers_finished(&q),
        queue_size(&q) == 0,
{
    crate::thread_safe_queue::ThreadSafeQueue::new(config)
}

// Spec function for active producer count
pub uninterp spec fn active_producer_count<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>) -> usize;

// Exec wrapper for active_producer_count
#[verifier::external_body]
#[verifier::when_used_as_spec(active_producer_count)]
pub fn active_producer_count_exec<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>) -> (count: usize)
    ensures count == active_producer_count(q)
{
    q.active_producer_count()
}

// Spec function for producers finished flag
pub uninterp spec fn producers_finished<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>) -> bool;

// Exec wrapper for producers_finished
#[verifier::external_body]
#[verifier::when_used_as_spec(producers_finished)]
pub fn producers_finished_exec<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>) -> (done: bool)
    ensures done == producers_finished(q)
{
    q.producers_finished()
}

// Spec function for queue size
pub uninterp spec fn queue_size<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>) -> usize;

// Exec wrapper for queue_size
#[verifier::external_body]
#[verifier::when_used_as_spec(queue_size)]
pub fn queue_size_exec<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>) -> (size: usize)
    ensures size == queue_size(q)
{
    q.queue_size()
}

// Increments active producer count
#[verifier::external_body]
pub fn register_producer<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>)
{
    q.register_producer()
}

// Decrements active producer count, sets done flag when reaching zero
#[verifier::external_body]
pub fn unregister_producer<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>)
{
    q.unregister_producer()
}

// Verify producer registration on new queue
fn verify_producer_registration() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    assert(active_producer_count(&q) == 0);
    assert(!producers_finished(&q));
    assert(queue_size(&q) == 0);
    
    register_producer(&q);
}

// Verify producer unregistration behavior
fn verify_producer_unregistration() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    assert(queue_size(&q) == 0);
    
    register_producer(&q);
    unregister_producer(&q);
}

}
