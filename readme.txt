For more information try --help
ken@ken-XPS-13-9360:~/rust/dinotree_inner$ cargo dinghy -d android bench
 INFO  cargo_dinghy > Targeting platform 'android-armv7' and device 'ZY223ZTPWZ'
    Finished release [optimized] target(s) in 0.0 secs

running 4 tests
test oned::selection_sort ... ignored
test dyntree::test::method1     ... bench: 101,573,260 ns/iter (+/- 18,968,776)
test dyntree::test::method_exp  ... bench:  70,281,569 ns/iter (+/- 14,750,988)
test dyntree::test::method_exp2 ... bench:  90,012,029 ns/iter (+/- 31,145,943)

test result: ok. 0 passed; 0 failed; 1 ignored; 3 measured; 0 filtered out

FORWARD_RESULT_TO_DINGHY_BECAUSE_ADB_DOES_NOT=0
WARNING: linker: Warning: unable to normalize ""
ken@ken-XPS-13-9360:~/rust/dinotree_inner$ cargo bench
    Finished release [optimized] target(s) in 0.0 secs
     Running target/release/deps/dinotree_inner-2044de1136e1f174

running 4 tests
test oned::selection_sort ... ignored
test dyntree::test::method1     ... bench:  18,070,890 ns/iter (+/- 6,989,769)
test dyntree::test::method_exp  ... bench:  13,522,272 ns/iter (+/- 318,582)
test dyntree::test::method_exp2 ... bench:  14,929,298 ns/iter (+/- 1,295,433)

test result: ok. 0 passed; 0 failed; 1 ignored; 3 measured; 0 filtered out
