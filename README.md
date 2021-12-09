# <img src="https://enzyme.mit.edu/logo.svg" width="75" align=left> The Enzyme High-Performance Automatic Differentiator of LLVM

This is a package containing a Rust frontend for [Enzyme](https://github.com/wsmoses/enzyme). This is very much a work in progress and bug reports/discussion is greatly appreciated!

Enzyme is a plugin that performs automatic differentiation (AD) of statically analyzable LLVM. It is highly-efficient and its ability perform AD on optimized code allows Enzyme to meet or exceed the performance of state-of-the-art AD tools.
  
# Supported types
- Scalars  
- Structs, Unions  
- Tuple, Array, Vec  
- Box, Reference, Raw pointer  

We are working on adding support for dyn trait objects, slices and enums.  
Adding Generics to your types or implementing traits is already working fine.


# Setup
First you have to get an adequate rustc/llvm/enzyme build here: [enzyme\_build](https://github.com/ZuseZ4/enzyme\_build).  
Afterwards for your convenience you should export this path for LLVM_SYS

```bash
$ export LLVM_SYS_130_PREFIX=$HOME/.config/enzyme/rustc-1.57.0-src/build/x86_64-unknown-linux-gnu/llvm  
```

and tell Enzyme about your library locations:  
```bash
$ export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$HOME/.config/enzyme/Enzyme-0.0.24/enzyme/build/Enzyme:$HOME/.config/enzyme/rustc-1.57.0-src/build/x86_64-unknown-linux-gnu/llvm/build/lib/  
```
  
As an alternative you can also run   
```bash
$ ninja install  
```
inside of your enzyme and llvm build directory.

Afterwards you can execute the following lines in `oxide-enzyme/example`, in order to compile the example.
```bash
$ cargo enzyme
```
You will find your executable in `./target/$TARGET/debug/`

# Compilation
We generate gradient functions based on LLVM-IR code. Therefore we currently need two compilation runs. The first to generate
a llvm-bc file with the LLVM-IR code, the second to process the bc file, generate the gradients, and build the entire crate.
You can do that manually using 
```bash
RUSTFLAGS="--emit=llvm-bc" cargo +enzyme -Z build-std rustc --target x86_64-unknown-linux-gnu -- --emit=llvm-bc -g -C opt-level=3 -Zno-link && RUSTFLAGS="--emit=llvm-bc" cargo +enzyme -Z build-std rustc --target x86_64-unknown-linux-gnu -- --emit=llvm-bc -g -C opt-level=3
```
We have created a wrapper for this command which you can call with:
```bash
cargo enzyme
```
Please be aware that our wrapper will ignore all additional commands.  
This approach won't work on dependencies since cargo doesn't support such a build process.



# FAQ  
- Q: How about Windows / Mac?
- A: WSL might work, the others probably not. Please let us know if you try.

  
# Further Information
More information on installing and using Enzyme directly (not through Rust) can be found on our website: [https://enzyme.mit.edu](https://enzyme.mit.edu).

To get involved or if you have questions, please join our [mailing list](https://groups.google.com/d/forum/enzyme-dev).

If using this code in an academic setting, please cite the following paper to appear in NeurIPS 2020

```
@inproceedings{NEURIPS2020_9332c513,
 author = {Moses, William and Churavy, Valentin},
 booktitle = {Advances in Neural Information Processing Systems},
 editor = {H. Larochelle and M. Ranzato and R. Hadsell and M. F. Balcan and H. Lin},
 pages = {12472--12485},
 publisher = {Curran Associates, Inc.},
 title = {Instead of Rewriting Foreign Code for Machine Learning, Automatically Synthesize Fast Gradients},
 url = {https://proceedings.neurips.cc/paper/2020/file/9332c513ef44b682e9347822c2e457ac-Paper.pdf},
 volume = {33},
 year = {2020}
}
```

License
=======

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
