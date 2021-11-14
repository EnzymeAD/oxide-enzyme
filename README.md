# <img src="https://enzyme.mit.edu/logo.svg" width="75" align=left> The Enzyme High-Performance Automatic Differentiator of LLVM

This is a package containing a Rust frontend for [Enzyme](https://github.com/wsmoses/enzyme). This is very much a work in progress and bug reports/discussion is greatly appreciated!

Enzyme is a plugin that performs automatic differentiation (AD) of statically analyzable LLVM. It is highly-efficient and its ability perform AD on optimized code allows Enzyme to meet or exceed the performance of state-of-the-art AD tools.
  

# Usage
First you have to get an adequate rustc/llvm/enzyme build here: [enzyme\_build](https://github.com/ZuseZ4/enzyme\_build).  
Afterwards for your convenience you should export this path for LLVM_SYS
> $ export LLVM_SYS_130_PREFIX=$HOME/.cache/enzyme/rustc-1.56.0-src/build/x86_64-unknown-linux-gnu/llvm  

and tell Enzyme about your library locations:  
> $ export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$HOME/.cache/enzyme/Enzyme-0.0.20/enzyme/build/Enzyme:$HOME/.cache/enzyme/rustc-1.56.0-src/build/x86_64-unknown-linux-gnu/llvm/build/lib/  
  
As an alternative you can also run   
> $ ninja install  

inside of your enzyme and llvm build directory.

Afterwards you can execute the following lines in `oxide-enzyme/example`
> $ cargo +enzyme run --release

# Supported types
- Scalars  
- Structs, Unions  
- Tuple, Array, Vec  
- Box, Reference, Raw pointer  

We are working on adding support for trait objects, slices and enums.

  
# FAQ  
- Q: How about Windows / Mac?
- A: It might work, please let us know if you had a chance to test it.

  
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

