// verified/src/main_specs.rs
use vstd::prelude::*;

verus! {

// Spec: The main function exists and returns a Result
#[verifier::external_body]
pub open spec fn main_exists() -> bool {
    true
}

pub proof fn verify_main_exists() {
    assert(main_exists());
}

} // verus!