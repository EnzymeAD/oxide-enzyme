# Oxide-Enzyme

This is a proof-of-concept work where we are trying to integrate [Enzyme](https://enzyme.mit.edu/Installation/) into Rust.  
Previous attempts were made at https://github.com/tiberiusferreira/oxide-enzyme and https://github.com/bytesnake/oxide-enzyme/
It's WIP, so please don't use it for any serious kind of work.
  
# Enzyme


# Usage
First you have to get an adequate rustc/llvm/enzyme build here: [enzyme\_build](https://github.com/ZuseZ4/enzyme\_build).  
Afterwards for your convenience you should export this path for LLVM_SYS
> $ export LLVM_SYS_120_PREFIX=$HOME/.config/enzyme/rustc-1.54.0-src/build/x86_64-unknown-linux-gnu/llvm  

and tell Enzyme about your library locations:  
> $ export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$HOME/.config/enzyme/Enzyme-0.0.16/enzyme/build/Enzyme:$HOME/.config/enzyme/rustc-1.54.0-src/build/x86_64-unknown-linux-gnu/llvm/build/lib/  
  
As an alternative you can also run   
> $ ninja install  

inside of your enzyme and llvm build directory.

Afterwards you can execute the following lines in `oxide-enzyme/example`
> $ cargo +enzyme run --release

# Working
Thanks to @cychen2021 we are supporting the following [types](https://doc.rust-lang.org/reference/types.html):
> Scalars  
> Sequence types: Tuple, Array, Slice*  
> User-defined types: Struct, Enum  
> Pointer types: References, Raw pointers  

*Slices should arrive during the next days.  
There is no fundamental issue with Function types, Trait types and Function pointers,  
we just haven't finished support yet.
  
# FAQ  
- Q: How about Windows / Mac?
- A: Should work soon, we need to fix some paths and test it.
