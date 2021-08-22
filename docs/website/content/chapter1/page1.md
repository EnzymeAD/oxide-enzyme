+++
title = "Oxide-Enzyme"
weight = 1
+++


This book is intended to 
Enzyme is a new tool for automatic differentiation with an awesome performance.
For those of you who can't wait to get started, here is your link: TODO


# About Automatic Differentiation

Enzyme differs from earlier tools in a few important aspects.
Usually tools have been developed by using Operator Overloading (OO) or by implementing a Source Code Transformation (SCT).

OO is easier to implement, but often suffered from being limited to overloaded operators (say no custom operator support).
Furthermore it is hard to optimize things on a larger level if you just have access to the operands.

SCT is usually more complex to implement, but also has more possibilities for optimizations.
If you know c you can probably imagine it as a kind of pre-processor. You have to mark certain code sections / functions / ...
and it will generate the required code to calculate the gradient. 
It gives authors the possibility to apply arbitrary optimizations before and after generating the gradient.
Optimizing the generated functions often isn't that relevant - the compiler will take care of it for us.
However, the first part is. Generally we will generate nice and simple gradient functions if we have a nice and simple primary function.
Having an unnecessary complex primary function might cause tools to generate a complex gradient function with dependencies that a compiler can't resolve anymore.
In their paper (TODO link) there is a simple example showing this issue.
SCT tools have to handle this issue by adding appropriate optimizations into their tools to make sure that primary functions will be simplified before 
generating the gradient. As you can imagine the list of possible optimizations is large and tools can therefore quickly become extremly complex.
We already have tools which are extremly good ad these optimizations, but they are part of compilers and therefore work on an intermediate representation (IR) 
of code, rather than on the source code itself.

Enzyme solved this issue by simply working on IR too. Now we have to compile our source code into IR, optimize it there, pass it to enzyme and 
optimize it a second time (just to make sure that our newly generated gradient functions are fast too).
This does not mean that there are no extra optimizations in Enzyme - there are! However, now there are on a much smaller scale.
There are a few situation where an optimiztion might not be that relevant for the primary function, so the compiler might refuse to apply it.
We could be smarter by knowing that it will result in generating better gradients, so we might decide to poke the compiler or add it directly to Enzyme.

Another (especially for us) important consequence is that Enzyme is able to generate for all languages, 
as long as we are able to compile it to the IR ( LLVM-IR to be specific).  \
That's easy: `cargo rustc -- -emit-llvm-bc`. Nice!

# Implementation

Here is a list of things I have done or intend to do to make Enzyme usage in rust a nice experience.

Done:
- Enable [llvm-plugins](https://github.com/rust-lang/rust/pull/86267) on nightly (Enzyme _is_ an llvm plugin, so this probably makes sense)
- Add [build flags](https://github.com/rust-lang/rust/pull/87297) to build llvm with it's plugin interface and clang
- Create [enzyme\_build](https://github.com/ZuseZ4/enzyme_build) which will build rustc/clang/enzyme/llvm with all the required flag's in the right configuration.
- Update [oxide-enzyme](https://github.com/rust-ml/oxide-enzyme). 
	- Strip broken build support from enzyme-sys. 
	- Remove Linux specific tools like objcopy and replace them with LLVM commands. 
	- Remove usage of OS specific folders like /usr/local/lib. We use a modified LLVM so our tools should not accidentally be linked against an existing llvm.
		After carefull renamining and adding more tests to feel more comfortable I might revert this to make use of the appropriate, system specific locations.
- Create two proc-macros. #[Enzyme] to mark primary functions and enzyme!(..) to call the gradient functions with the desired settings.

# Installation / setup

git clone https://github.com/ZuseZ4/enzyme\_build
cd enzyme\_build
cargo run --release
This might take a few hours, depending on your cpu. 
We are working on simplifying this.

# Usage

# Warnings 
Enzyme is rapidly evolving and so are we. Expect breaking changes and other calling conventions if the current ones turn out to be uneccessary complex 
(e.g. I intend to remove the need for #[Enzyme]).

# Next goals
- Rayon support. Enzyme already does support various types of parallelism so this is a comparably small step. 
- Speed up installation by adding support for downloading pre-compiled rustc/llvm/enzyme.
- Use (llvm based) GPGPU Kernels written in Rust with Enzyme.
- Download the release tarballs instead of cloning the rust repository using git once 1.56 hits stable. Reduces dependencies further and is a little bit faster.
- Test all the extra combinations which user might try (enzyme + cross-compilation / no-std / x)
- Get Rust into HPC :)


Pages and sections are actually very similar.

## Page variables
Gutenberg will try to load the `templates/page.html` template, the `page.html` template of the theme if one is used
or will render the built-in template: a blank page.

Whichever template you decide to render, you will get a `page` variable in your template
with the following fields:


```ts
content: String;
title: String?;
description: String?;
date: String?;
slug: String;
path: String;
permalink: String;
summary: String?;
tags: Array<String>;
category: String?;
extra: HashMap<String, Any>;
// Naive word count, will not work for languages without whitespace
word_count: Number;
// Based on https://help.medium.com/hc/en-us/articles/214991667-Read-time
reading_time: Number;
// `previous` and `next` are only filled if the content can be sorted
previous: Page?;
next: Page?;
// See the Table of contents section below for more details
toc: Array<Header>;
```

## Section variables
By default, Gutenberg will try to load `templates/section.html`. If there isn't
one, it will render the built-in template: a blank page.

Whichever template you decide to render, you will get a `section` variable in your template
with the following fields:


```ts
content: String;
title: String?;
description: String?;
date: String?;
slug: String;
path: String;
permalink: String;
extra: HashMap<String, Any>;
// Pages directly in this section, sorted if asked
pages: Array<Pages>;
// Direct subsections to this section, sorted by subsections weight
subsections: Array<Section>;
// Naive word count, will not work for languages without whitespace
word_count: Number;
// Based on https://help.medium.com/hc/en-us/articles/214991667-Read-time
reading_time: Number;
// See the Table of contents section below for more details
toc: Array<Header>;
```

## Table of contents

Both page and section have a `toc` field which corresponds to an array of `Header`.
A `Header` has the following fields:

```ts
// The hX level
level: 1 | 2 | 3 | 4 | 5 | 6;
// The generated slug id
id: String;
// The text of the header
title: String;
// A link pointing directly to the header, using the inserted anchor
permalink: String;
// All lower level headers below this header
children: Array<Header>;
```

