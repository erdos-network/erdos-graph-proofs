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
    ensures
        active_producer_count(q) > 0,
{
    q.register_producer()
}

// Decrements active producer count, sets done flag when reaching zero
#[verifier::external_body]
pub fn unregister_producer<T>(q: &crate::thread_safe_queue::ThreadSafeQueue<T>)
{
    q.unregister_producer()
}

// Verify producer registration increments count
fn verify_producer_registration() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    assert(active_producer_count(&q) == 0);
    assert(!producers_finished(&q));
    assert(queue_size(&q) == 0);
    
    let old_count = active_producer_count_exec(&q);
    register_producer(&q);
    let new_count = active_producer_count_exec(&q);
    
    assert(new_count == old_count + 1);
}

// Verify producer unregistration decrements count and sets done flag
fn verify_producer_unregistration() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    assert(queue_size(&q) == 0);
    
    register_producer(&q);
    let old_count = active_producer_count_exec(&q);
    unregister_producer(&q);
    let new_count = active_producer_count_exec(&q);
    
    assert(new_count == old_count - 1);
    assert(new_count == 0);
    assert(producers_finished_exec(&q));
}

// Verify multiple producer registration
fn verify_multiple_producers() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    assert(active_producer_count_exec(&q) == 0);
    
    register_producer(&q);
    assert(active_producer_count_exec(&q) == 1);
    
    register_producer(&q);
    assert(active_producer_count_exec(&q) == 2);
    
    register_producer(&q);
    assert(active_producer_count_exec(&q) == 3);
}

// Verify partial unregistration doesn't set done flag
fn verify_partial_unregistration() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    register_producer(&q);
    register_producer(&q);
    assert(active_producer_count_exec(&q) == 2);
    
    unregister_producer(&q);
    assert(active_producer_count_exec(&q) == 1);
    assert(!producers_finished_exec(&q));
    
    unregister_producer(&q);
    assert(active_producer_count_exec(&q) == 0);
    assert(producers_finished_exec(&q));
}

// Verify unregister saturates at zero
fn verify_unregister_saturates() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    assert(active_producer_count_exec(&q) == 0);
    
    unregister_producer(&q);
    assert(active_producer_count_exec(&q) == 0);
}

// Verify register doesn't affect queue size (state isolation)
fn verify_register_preserves_queue_size() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    let size_before = queue_size_exec(&q);
    register_producer(&q);
    let size_after = queue_size_exec(&q);
    
    assert(size_before == size_after);
}

// Verify unregister doesn't affect queue size (state isolation)
fn verify_unregister_preserves_queue_size() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    register_producer(&q);
    
    let size_before = queue_size_exec(&q);
    unregister_producer(&q);
    let size_after = queue_size_exec(&q);
    
    assert(size_before == size_after);
}

// Verify sequential operations maintain consistency (lock release/reacquire)
fn verify_sequential_consistency() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    // First operation
    register_producer(&q);
    let count1 = active_producer_count_exec(&q);
    assert(count1 == 1);
    
    // Second operation (requires lock release and reacquire)
    register_producer(&q);
    let count2 = active_producer_count_exec(&q);
    assert(count2 == 2);
    
    // Third operation
    unregister_producer(&q);
    let count3 = active_producer_count_exec(&q);
    assert(count3 == 1);
    
    // Fourth operation
    unregister_producer(&q);
    let count4 = active_producer_count_exec(&q);
    assert(count4 == 0);
    assert(producers_finished_exec(&q));
}

// Verify consistent reads (no torn reads)
fn verify_consistent_reads() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    register_producer(&q);
    register_producer(&q);
    
    // Multiple reads should give same result
    let count_a = active_producer_count_exec(&q);
    let count_b = active_producer_count_exec(&q);
    let count_c = active_producer_count_exec(&q);
    
    assert(count_a == count_b);
    assert(count_b == count_c);
    assert(count_a == 2);
}

// Verify atomic state transitions
fn verify_atomic_transitions() {
    let config = default_queue_config();
    let q: crate::thread_safe_queue::ThreadSafeQueue<u32> = new_queue(config);
    
    register_producer(&q);
    
    // State before
    let count_before = active_producer_count_exec(&q);
    let finished_before = producers_finished_exec(&q);
    let size_before = queue_size_exec(&q);
    
    // Atomic operation
    unregister_producer(&q);
    
    // State after - all fields updated atomically
    let count_after = active_producer_count_exec(&q);
    let finished_after = producers_finished_exec(&q);
    let size_after = queue_size_exec(&q);
    
    // Verify atomic transition
    assert(count_before == 1);
    assert(!finished_before);
    assert(count_after == 0);
    assert(finished_after);
    assert(size_before == size_after);
}

}
