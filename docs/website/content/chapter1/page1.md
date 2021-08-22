+++
title = "Oxide-Enzyme"
weight = 1
+++


This book is intended to 
Enzyme is a new tool for automatic differentiation with an awesome performance.
For those of you who can't wait to get started, here is your link: TODO




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

![usage](https://github.com/ZuseZ4/oxide-enzyme/blob/master/README.md)
