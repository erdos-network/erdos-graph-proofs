.PHONY: verify clean

verify:
	verus --crate-type=lib verified/src/lib.rs

clean:
	rm -rf verified/target
