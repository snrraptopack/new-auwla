# Extension API Documentation (Comprehensive)

This document is a full reference for Auwla extension APIs.

It covers:

1. Extension syntax and declaration forms
2. Method signatures and parameter semantics
3. Generic extension design
4. Overload resolution rules
5. Attribute system for extension methods
6. JavaScript interop via method-level external mappings
7. Type-level APIs via type declarations
8. Operator extension model
9. Module/import behavior for extensions
10. Error and ambiguity behavior
11. API design recommendations

## Scope and non-goals

This is about the extension API model itself.

It is not a full list of every std method name.

Important:

- No existing std extension functions were removed by these docs.
- This guide explains how the system works and how to use it correctly.

## 1) Core concepts

Auwla supports three related API surfaces:

1. Top-level functions
2. Extensions on types using extend blocks
3. Type-level method containers using type declarations

The extension API mainly means 2 and 3.

## 2) Extend declaration forms

## 2.1 Basic extend

```auwla
extend number {
    fn double(self): number => self * 2;
}
```

Meaning:

- Target type: number
- Adds instance method double

## 2.2 Generic extend

```auwla
extend <T> array<T> {
    fn first(self): T? => self.get(0);
}
```

Meaning:

- Declares generic parameter T at extend block level
- Applies to array<T>

## 2.3 Composite type targets

You can target wrappers and structural forms, for example:

- optional-like T?
- result-like T?E
- array<T>
- dict<K, V>

Example:

```auwla
extend <T, E> T?E {
    fn is_ok(self): bool {
        return match self {
            some(_) => true,
            none(_) => false,
        };
    }
}
```

## 3) Method declaration model

Method syntax in extend supports:

1. Instance and static methods
2. Generic method params
3. Optional annotations for params and returns
4. Block body, arrow body, or ambient semicolon declaration
5. Attributes

## 3.1 Instance method

```auwla
fn join_with(self, sep: string): string
```

Call:

```auwla
value.join_with(",")
```

## 3.2 Static method

```auwla
static fn from_code(code: number): string
```

Call:

```auwla
string::from_code(65)
```

## 3.3 Generic method inside extend

```auwla
extend <T> array<T> {
    fn map_into<U>(self, f: (T) => U): array<U> {
        // body omitted
    }
}
```

Type parameter set used by a method is the union of:

1. Extend-level type parameters
2. Method-level type parameters

## 3.4 Ambient extension methods

You can declare methods with semicolon when the implementation is external:

```auwla
extend string {
    @external("js", "method", "toUpperCase")
    fn to_upper(self): string;
}
```

Ambient means:

- Signature exists for typechecking
- No Auwla body is required

## 4) Parameter semantics

Each parameter slot has:

1. Name
2. Type
3. Is-vararg flag

## 4.1 self handling

For instance methods:

- self is expected as the receiver parameter
- self type is resolved from extend target type

For static methods:

- no receiver value
- call with TypeName::method(...)

## 4.2 Varargs

A vararg parameter is declared with ellipsis.

Conceptually:

```auwla
fn add_many(self, others: number...): number
```

Behavior:

- Minimum required args are all fixed params before vararg
- Remaining call args are matched against vararg element type

## 4.3 Return type behavior

If omitted, return defaults to void-like behavior.

For strict APIs, always annotate return type.

## 5) Generic semantics in extensions

## 5.1 Explicit generic declaration

Extend generics are explicit:

```auwla
extend <T> array<T> { ... }
```

Method generics are explicit:

```auwla
fn identity<U>(self, v: U): U { ... }
```

## 5.2 Generic substitution points

Generic substitution applies in:

1. Method params
2. Method return
3. Receiver self type matching
4. Static calls with type arguments

## 5.3 Constraint attribute (current mechanism)

Current constraint mechanism is attribute-based:

```auwla
@constraint("T", "number")
fn only_num<T>(self, v: T): number => 1;
```

Meaning:

- T must resolve to number for this call

Current constraint type literals supported by checker include:

- number, string, bool, char, void, array, dict

## 6) Overload resolution rules

Auwla supports multiple methods with the same name.

Resolution model:

1. Gather candidate methods by method name and receiver/static context
2. Try typechecking each candidate against call args
3. Keep successful candidates
4. Choose best scored candidate
5. If multiple best candidates are truly distinct, report ambiguity
6. If ties are semantically identical duplicates (import aggregation), dedupe and proceed

Practical guidance:

- Prefer clearly different signatures
- Avoid near-duplicate generic overloads that differ only by type param names

## 7) Attribute system for extension APIs

Attributes are attached to methods and can influence semantics.

Common extension-related attributes:

1. external for host interop
2. constraint for generic constraints

## 7.1 Method-level external mappings

External mapping is declared on method:

```auwla
@external("js", kind, ...)
fn ...
```

Common kinds in extension methods:

1. property
2. method
3. static

Generated behavior differs by call site:

- instance method calls may inline JS property/method access
- static calls may inline namespace/class style JS calls

## 7.2 Optional/result wrapping for external methods

For external methods, return wrappers are applied when return type is optional/result.

1. Optional return

- null/undefined maps to none
- value maps to some(value)

2. Result return

- try/catch wrapper
- success to some(value)
- failure to none(error)

## 8) Type declarations as API containers

Auwla also supports type-level method declarations:

```auwla
@external("namespace")
type Math {
    @external("js", "method", "floor")
    static fn round_down(x: number): number;
}
```

This gives namespaced API style:

```auwla
let n = Math::round_down(3.9);
```

Type declarations can be used for:

1. Namespace-like APIs
2. External class/static facades
3. Grouped static utilities

## 9) Operator extension API

Operators are implemented as methods with operator syntax.

Example:

```auwla
extend Vec2 {
    operator +(self, other: Vec2): Vec2 {
        return Vec2 { x: self.x + other.x, y: self.y + other.y };
    }
}
```

Supported operator categories include arithmetic and range operators.

Guideline:

- Keep operator semantics unsurprising and consistent with type intent.

## 10) Module and import behavior for extensions

Extensions are treated as globally available survivors once brought into compilation context.

In module compilation flow:

1. Imported module exports are pre-collected
2. Extension registries from imported modules are merged
3. Calls resolve against merged extension set

This is why duplicate semantic registrations can occur across import aggregation, and why dedupe exists.

## 11) Call resolution surfaces

Extension dispatch is applied on:

1. Instance method calls
2. Static method calls
3. Some property access patterns via external property mappings

If no extension candidate matches, checker reports method not found or overload mismatch.

## 12) Error behavior you should expect

Common error classes:

1. Method not found on receiver type
2. Static versus instance misuse
3. Wrong argument count
4. Generic type argument arity mismatch
5. Constraint violations
6. Ambiguous overload call

Use explicit types and fewer ambiguous overloads to improve diagnostics.

## 13) Design patterns (recommended)

## 13.1 Keep extension families coherent

Group methods by target type responsibility.

Example:

- array<T> methods for sequence operations
- dict<K, V> methods for key/value operations

## 13.2 Keep naming action-oriented

Prefer clear verbs:

- get
- set
- map
- filter
- parse
- from_x

## 13.3 Keep receiver-first mental model

For instance methods:

- self is conceptually the object being transformed

For external method mapping:

- first param maps to receiver object in host call

## 13.4 Prefer explicit return types in public APIs

Especially for:

1. External interop methods
2. Generic methods
3. Operator methods

## 13.5 Use optional/result intentionally

Use optional when absence is normal.
Use result when failure reason matters.

## 14) End-to-end example combining extension features

```auwla
extend <T> array<T> {
    fn safe_last(self): T? {
        if self.len() == 0 {
            return none;
        }
        return self.get(self.len() - 1);
    }
}

extend number {
    @constraint("T", "number")
    fn add_t<T>(self, other: T): number {
        return self + other;
    }
}

extend string {
    @external("js", "method", "toUpperCase")
    fn to_upper(self): string;
}

@external("namespace")
type Math {
    @external("js", "method", "max")
    static fn max(a: number, b: number): number;
}

fn main() {
    let xs = [1, 2, 3];

    match xs.safe_last() {
        some(v) => print("last={v}"),
        none => print("empty"),
    }

    let s = "auwla".to_upper();
    let m = Math::max(4, 9);

    print("{s} / {m}");
}
```

## 15) Standalone global external functions (related, not extension-only)

Standalone external functions are also available:

```auwla
@external("js", "function", "__print")
fn print(msg: string): void;
```

This is separate from extension methods but interoperates naturally with extension-based APIs.
