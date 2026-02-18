use vstd::prelude::*;

verus! {

pub struct HelixGraphEngine {}

pub uninterp spec fn node_count(engine: &HelixGraphEngine) -> usize;

pub uninterp spec fn edge_count(engine: &HelixGraphEngine) -> usize;

#[verifier::external_body]
pub fn new_engine() -> (e: HelixGraphEngine)
    ensures
        node_count(&e) == 0,
        edge_count(&e) == 0,
{
    unimplemented!()
}

fn check_new_engine_empty() {
    let e = new_engine();
    assert(node_count(&e) == 0);
    assert(edge_count(&e) == 0);
}

}
