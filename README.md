# WESL for Rust

**A work in progress!**

This project will implement the necessary tools to build complex shaders, like [naga_oil](https://github.com/bevyengine/naga_oil) does for [bevy](https://bevyengine.org/), but in a framework-agnostic way. At [wgsl-tooling-wg](https://github.com/wgsl-tooling-wg), we aim at standardizing a few language extensions to facilitate the work of engine developers and shader witches.

## Goals

* *Correct*, mirror concepts present in [the wgsl spec](https://www.w3.org/TR/WGSL/).
* *Flexible*, allow extending the wgsl syntax with well-defined extensions (see [wgsl-tooling-wg](https://github.com/wgsl-tooling-wg)).
* *User-friendly*, by sticking to the spec as much as possible, by providing clear and well-documented interfaces.

## Non-goals

* *Performant*, we care about correctness, less so about time and memory constraints. The interface must be as user-friendly as possible. These tools target offline compilation first.
