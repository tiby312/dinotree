Provides the dinotree data structure and ways to traverse it. All divide and conquer style query algorithms that you can do on this tree would be done using the Vistr nd VistrMut visitors. No actual query algorithms are provided in this crate. Only the data structure and a way to construct it are provided in this crate.

Requires nightly rust to use these features:
~~~~text
#![feature(ptr_internals)]
#![feature(align_offset)]
#![feature(trusted_len)]
#![feature(test)]
~~~~
