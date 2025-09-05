# MEW = Modular Extensions for WGSL

## Introduction

MEW is a strict super-set of the [WebGPU Shader Language](https://www.w3.org/TR/WGSL) (WGSL).

The language aims to have a well considered feature set that maintains the spirit of the original design, while adding powerful abstractions that enable programming in the large.

## Features

### Modules

In addition to variables, structs and function
declarations, MEW adds the concept of _modules_.

When you create a file in MEW, it is automatically
a module, identified by its name. In addition, you
can declare inline modules using the `module` keyword.

Inline modules can contain almost everything a file
module, with the exception of the enable, requires and diagnostic directives.

The two ways of declaring modules are treated interchangeably, which means that consumers of a particular module need not distinguish between whether it was implemented as a file, or inline.

Module and their contents can be accessed by using their fully qualified path, [import statements](<README#Import Statements>), or [aliases](<README#Aliasing Everything>).

### Generics

Inline modules and function declarations can accept generic arguments.

Generic arguments can be other modules, types, or constant values. Like classes in other languages, module specialization occurs for each unique set of generic parameter.

Declaring that a module or function accepts generic arguments uses the familiar angle bracket `< >` syntax.

Users can supply generic parameters to generic members using angle brackets, or by providing an anonymous module after the path using the `with` keyword. Arguments can be _named_, or positional.

### Inline pathing

Inline pathing allows users to refer to symbols by their module path, for example:

```wgsl
fn cool_fn() -> Hello::PopulatedWorld<I32>::t {
    let a: Hello::World<I32>::t = Hello::World<I32>::make_world(10);
    return Hello::World<I32>::populate(a);
}
```

### Import Statements

`import` statements bring imported symbols into scope, as well as allowing for compact aliasing of imported symbols using the `as` keyword. Their primary purpose is to make module use more ergonomic. `import` can be used in module, function and block scopes.

Rewriting the example in [Inline Pathing](<README#Inline pathing>):

```wgsl
import Hello::{
    World<I32> as World,
    PopulatedWorld<I32>::t as populated_world
};

fn cool_fn() -> populated_world {
    import World::{ t as world, make_world, populate };

    let a: world = make_world(10);
    return populate(a);
}
```

### Aliasing Everything

All symbols (types, modules, constants, and functions) can be aliased in the file or module using the already existing `alias` keyword. This allows users to re-export symbols from other modules, and when used as a generic argument, allow modules to delegate implementation of a required module member to another symbol.

```wgsl
alias type_alias = f32; // A type
alias module_alias = MyModule; // A module
alias function_alias = cool_func; // A function
alias constant_alias = Math::PI; // A constant
```

### Anonymous modules & the `with` keyword

Generics can take in multiple arguments and sometimes the arguments are modules or functions that are highly specific to the module. To solve this problem, imports and usages can use `with`. `with` is followed by an anonymous module definition. For example:

```wgsl
import SmallSet with {
    alias t = i32;
    fn hash_func(a: i32) -> i32 {
        return a;
    }
} as I32Set;

fn make_small_set(size: u32) -> I32Set::t {
    return I32Set::make(size);
}
```

### `extends` keyword

The `extends` keyword allows users to compose behavior from other modules. It does this by creating
aliases for each module member in the current module.

The behavior of `extends` allows module composition without bloating output with identical symbols, though does have implications for stateful module members.

```wgsl
module BaseModule {
    fn handy_fn() -> HandyStruct
    {
        // impl here
    }
}

module DerivedModule {
    extend BaseModule;
}

// Note that DerivedModule::handy_fn has the same
// identity as BaseModule::handy_fn.
```

## Example MEW Code

Please see the [test folder](./crates/mew-test/) for examples of MEW shader code.

## Future Features

These are in no particular order, may be subsumed by WESL community efforts, and make no guarantees about whether these will actually be done, nevertheless, here are some ideas for future efforts:

### Lazy module loading

Currently `mew-api` requires that the text of all source files be added to the translation unit. This creates a chicken and egg problem in practical use as there is no way to introspect on what modules are in use for a given entrypoint. The solution to this is to forward declare the locations of the source files in `mew-api` and then only load them if they are being referenced.

### Compliance w/ WESL import spec

The current version of MEW was created before the [Community Standard for Enhanced WGSL](https://wesl-lang.dev) (WESL) [import spec](https://wesl-lang.dev/spec/Imports) was finalized. This means that the import behavior is different to that of the specification.

As the [Relationship between WESL and MEW](<README#Relationship between WESL and MEW>) section explains, MEW aims to not only be a super-set of WGSL but of WESL, which means this divergence should be corrected.

### Module Interfaces and Type Field Sets

Currently there is no way to constrain generic arguments or to provide information hiding.

A proposed design is to create support for creating module interfaces that allow one to specify the structural constraints on a module (with possible default implementations) and analogously, a required field set would allow one to require expected fields on a given `struct` without reference to specific layout (similar to TypeScript).

This is a relatively high effort endeavor. Adding typechecking will entail reproducing a spec-compliant WGSL check as well as a structural typecheck and constraint solver. However this is a necessary push to make MEW a practical language.

### Compliance w/ WESL Conditional Translation Spec

MEW does not currently have the ability to perform conditional compilation.

As the [Relationship between WESL and MEW](<README#Relationship between WESL and MEW>) section explains, MEW aims to not only be a super-set of WGSL but of WESL, which means this feature should be included in MEW.

```wgsl
struct Foo {
  a: f32,
  @if(some_condition)
  b: f32,
  @else
  b: u32
}
```

### Probably not any time soon: Generators

Conditional compilation can only go so far. Generators
are a concept where a succession of contextually well-formed syntax elements are generated. This allows one to procedurally generate modules, structs and functions in a way that can be analyzed for correctness and thus retain the benefits of language servers.

```wgsl
module* FibonacciGenerator<const length: u32> {
    let prev: u32 = 0;
    let current: u32 = 1;
    if(length > 0) {
        for (let i = 2; i < length; ++i) {
            let temp = f0;
            f0 = f1;
            f1 = temp + f0;
        }
        prev = current;
    }
    yield const result: u32 = prev;
}

struct* Vec_Generator<t: type, const size: u32> {
    for(let i = 0; i<size; ++i) {
        yield `element_${i}`: t,
    }
}
```

### Struct `extends`

Modules currently can be extending using `extends`. However allowing composition is useful for structs too. The Struct `extends` feature will allow users to extend another type by calling `extends` in the struct member list.

```wgsl
struct A {
    a: f32,
    b: array<vec4<f32>>
}

struct BadB {
    extend A, // illegal, A has a runtime array so must be last
    c: f32
}

struct B {
    c: f32,
    extend A, // allowed
}
```

### `new` keyword

The `extends` and `alias` keywords currently do not create a new instance of a module. They simply reference the existing symbols of the base module.
A `new` keyword would allow users to extend a module by performing a member-wise copy of module members rather than simply creating an alias to the base symbols.

```wgsl
// assert(identity(AliasedModule::c) == identity(CoolModule::c))
// CoolModule and AliasedModule members have
// exactly the same identity
alias AliasedModule = CoolModule;

// assert(identity(NewModule::c) != identity(CoolModule::c))
// CoolModule and NewModule members have the same values, but their identities are different
alias NewModule = new CoolModule;
```

### Source Maps

Though MEW attempts to produce predictable (somewhat) human-readable output code, for building debug tooling and error reporting, it is important to provide [source maps](https://developer.mozilla.org/en-US/docs/Glossary/Source_map) that enable tools to correlate generated output code with the original inputs.

### Introspection

Many use cases rely on shader introspection to generate or interact with host code. Having a robust introspection API would be advantageous to cater for this use case.

### Syntax Highlighting Support

Syntax highlighting support is a baseline expectation
for all languages, and MEW is no exception.

[TextMate's Language Grammar](https://macromates.com/manual/en/language_grammars) is a very common format for syntax highlighters used by text-editors, while [Tree-sitter](https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html?highlight=inject) is increasingly being used for highlighting. Creating grammars for one or both would result in high-coverage support for highlighting.

### Language Server Support

Having at least basic language server support would make MEW a much more viable language.

### Better Documentation

An obvious but often overlooked barrier to adoption.

### Function Overload Support

In MEW as it stands, symbols are shadowed. This means that the inner scope can hide symbols with the same name in the outer scope. However, wgsl supports function _overloads_ which instead attempts to find the first function on any level for which the arguments match.

### Module Inference

One complaint with the design of the module system is that it isn't very conducive to inference, adding a type checker will not on its own entirely solve this problem, however would possibly allow module inference. To understand the complaint, let us look at this simple example:

```wgsl

const numbers = array<i32, 7>(1,1,2,3,5,8,13);

fn sum<N: Numeric, A: Array<N>>(arr: A::t) {
    import N::(operator+);
    let result: N::t = N::ZERO;
    for(let i=0; i < A::arrayLength(&arr); ++i) {
        result += arr[i];
    }
    return result;
}

const result = sum<I32, FixedArray<I32, 7>>(numbers);
```

In the example above, the generic parameter of `sum` had to be fully specified. We already know that numbers is `array<i32, 7>`, but we could not infer how `array<i32, 7>` related to `FixedArray<I32, 7>` (and thus `Array<N::t>`) and how `i32` related to `I32` (and thus `Numeric`).

Module inference could improve matters by examining the modules that are in scope in the current context, and by constructing a system of constraints based on arguments passed to a function, produce a narrowed set of modules that satisfy the constraints. If only one qualifies, then inference succeeds.

Module inference is particularly critical for these generic operations on numeric types. If for arguments sake we introduce a standard library containing modules with operations on built-in types `I32`, `Vec3<f32>`, `FixedArray<T, const size: i32>` etc and automatically add them to the outermost scope, we can turn the example above into simply:

```wgsl
const numbers = array<i32, 7>(1,1,2,3,5,8,13);
const result = sum(numbers);
```

We have a system of terms:

| Terms                  |
| ---------------------- |
| `A::t = array<i32, 7>` |
| `A satisfies Array<Y>` |
| `Y = N::t`             |
| `N satisfies Numeric`  |
| `A::element = Y`       |

We can immediately reduce this set by rewriting some terms:

| Terms                     |
| ------------------------- |
| `A::t = array<i32, 7>`    |
| `A satisfies Array<N::t>` |
| `N satisfies Numeric`     |
| `A::element = N::t`       |

Both `RuntimeArray<Numeric::t>` and `FixedArray<Numeric::t, _>` satisfy `Array<Numeric::t>`. However the type `t` of `RuntimeArray<Numeric::t>` is `array<Numeric::t>` and thus `RuntimeArray` is immediately ruled out as it cannot match the constraints.

This leaves us with `FixedArray<Numeric::t, _>`. Partially resolving `FixedArray<Numeric::t, _>`, gives us `alias t = array<Numeric::t, _>`. In this case, `t` _does_ match `array<i32, 7>`, and `element = i32` matches too, and so we can infer that `Numeric::t` is `i32`. `I32` is the only `Numeric` module in scope that has `t: i32` and so we can infer that `const result = sum(numbers)` is `const result = sum<I32, FixedArray<I32, 7>>(numbers)`.

### Standard Library

To support inference and to prevent proliferation of core libraries, a small set of core types could be automatically added to the global scope, including numeric types and array types.

### Type to Module Coercion

To reduce instances of users having to create anonymous modules containing a single type (by convention): `t`, and to simplify the generics system, it might be wise to sugar type arguments to an module that assigns the type to `t` as an alias.

## Relationship between WESL and MEW

The WGSL language is conspicuous in its absence of many niceties expected of shader languages and programming languages in general. This has led to a proliferation of mutually incompatible language extensions to WGSL. This means that it is extremely difficult to share shader code between projects but also means that the ecosystem suffers due to primitive tooling due to lack of an agreed-upon standard for features such as conditional compilation and modularization.

WESL is a grass-roots attempt to develop a standard that a number of compliant implementations can gather around, allowing for code-sharing and improved tooling.

MEW aims to be a super-set not only of WGSL but also of WESL. In a sense, its purpose is as a laboratory for exploring what a module-oriented shader language could look like, hopefully acting as a vanguard for further WESL features.
