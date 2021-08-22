+++
title = "Building from scratch"
weight = 2
+++

I recommend to have a look at the installation instructions for [enzyme](https://enzyme.mit.edu/Installation/).  
There are a few things to consider here:  
1) Enzyme requires some symbols which are not exposed for the default stable/beta/nightly toolchain. We can use the configure argument "--enable-llvm-link-shared" to 
build an LLVM along with rustc which exposes the required symbols.
2) The safest solution is building Enzyme and rustc based on the same llvm version. Enzyme does not support LLVM-13 yet (20.8.2021) while
Rust recently moved to LLVM-13. Therefore I recommend using the latest LLVM-12 [commit](https://github.com/rust-lang/rust/commit/3cfb7305ddb7fd73b92c87ae6af1b169068b6b0f).  
3) For a simpler download we can use the release tarballs of enzyme and rust. For now we are using the build flags "--enable-clang" and "--enable-lld" which are currently only 
available in the github repository. They will probably be part of the release tarballs starting with the Rust version 1.56.
