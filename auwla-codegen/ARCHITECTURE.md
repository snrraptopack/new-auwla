# Auwla Codegen — Architecture Guide

> **Purpose**: This document explains the core architecture of the Auwla code generator to help any developer (human or AI) understand, navigate, and extend it safely.

---

## High-Level Pipeline

```
Source (.aw)  →  Lexer  →  Parser  →  AST  →  Typechecker  →  Codegen  →  JavaScript
```

| Crate | Role |
|-------|------|
| `auwla-lexer` | Tokenizes `.aw` source code into a token stream |
| `auwla-parser` | Parses tokens into an AST (`Program` = list of `Stmt`) |
| `auwla-ast` | Defines the shared AST types: `Stmt`, `Expr`, `Type`, `Pattern`, etc. |
| `auwla-typechecker` | Validates types, resolves extensions/enums, collects exports |
| **`auwla-codegen`** | Emits JavaScript from the typed AST |
| `auwla-cli` | Orchestrates the pipeline: parsing, typechecking, codegen, I/O |
| `auwla-error` | Pretty-prints diagnostic errors (like rustc) |

---

## Codegen Module Structure

```
auwla-codegen/src/
├── lib.rs          # Public API: re-exports emit_js() and postprocess module
├── writer.rs       # CodeWriter — low-level indented string buffer
├── emitter.rs      # JsEmitter struct — central state + utility methods
├── stmt.rs         # Statement emission (let, var, fn, if, for, extend, etc.)
├── expr.rs         # Expression emission (literals, calls, match, closures, etc.)
├── match_logic.rs  # Match expression/statement emission (if-else and switch)
├── pattern.rs      # Pattern matching conditions + variable binding
├── try_logic.rs    # Try operator emission (try-as-assign, try-as-expr, standalone)
├── external.rs     # @external(js, ...) attribute handling for FFI methods
└── postprocess.rs  # Post-emission JS fixups (import prefixing for __util, __runtime)
```

### Entry Point

```rust
// lib.rs
pub fn emit_js(program, extensions, enums) -> (String, String)
//                                             ↑ main JS    ↑ extensions JS (__runtime.js)
```

The CLI calls `emit_js()` to get the generated JS, then calls `postprocess::add_runtime_imports()` to prepend any required `import` statements.

---

## Key Types

### `CodeWriter` (writer.rs)
Low-level buffer managing indentation. Both main output and extension output use their own `CodeWriter` instance.

```rust
pub struct CodeWriter {
    buffer: String,
    indent: usize,
}
```

### `JsEmitter` (emitter.rs)
Central state struct created per-file compilation. **All codegen modules implement methods on this struct** via Rust's `impl JsEmitter` in separate files.

```rust
pub struct JsEmitter {
    out: CodeWriter,          // Main JS output buffer
    ext: CodeWriter,          // Extensions output buffer (__runtime.js)
    temp_counter: usize,      // Unique temp vars: __match_0, __match_1, ...
    var_types: HashMap,       // name → type key (for extension method dispatch)
    ext_methods: HashMap,     // type → set of method names (fast lookup)
    extensions: HashMap,      // Full extension signatures
    enums: HashSet,           // Known enum type names
    in_extension_method: bool, // self→__self rewriting active
    is_statement_context: bool,// Suppresses return injection
}
```

### Two Output Buffers
The emitter writes to **two separate buffers**:
1. **`out`** — The main per-file JS output (functions, variables, control flow)
2. **`ext`** — Extension method definitions (`_ext_TypeName_method(...)`) that get combined into `__runtime.js`

Use `self.write()` / `self.writeln()` for main output, and `self.write_ext()` / `self.writeln_ext()` for extension output.

---

## How Auwla Constructs Map to JavaScript

| Auwla | JavaScript |
|-------|-----------|
| `let x = 5` | `const x = 5;` |
| `var x = 5` | `let x = 5;` |
| `fn foo(a) { ... }` | `function foo(a) { ... }` |
| `print("hi")` | `__print("hi")` (custom runtime function) |
| `1..10` range | `__range(1, 10, false)` (runtime helper) |
| `some(val)` | `{ ok: true, value: val }` |
| `none` / `none(err)` | `{ ok: false }` / `{ ok: false, value: err }` |
| `Enum::Variant(data)` | `{ $variant: "Variant", $data: [data] }` |
| `expr.method(args)` on extended type | `_ext_TypeName_method(expr, args)` |
| `Type::static_method(args)` | `_ext_TypeName_static_method(args)` |
| `struct Name { ... }` | No-op (types vanish at runtime) |
| `enum Name { ... }` | No-op (types vanish at runtime) |
| `match expr { ... }` as expression | IIFE: `(() => { ... })()` |
| `match expr { ... }` as statement | Inline if-else chain or `switch` |
| `import { x } from './mod'` | `import { x } from './mod.js';` |
| `export fn foo() {}` | `export function foo() {}` |

---

## Key Design Decisions

### 1. Extension Methods are Free Functions
Auwla's `extend TypeName { fn method(self) { ... } }` compiles to standalone `_ext_TypeName_method(__self, ...)` functions. These live in a shared `__runtime.js` file and are imported globally.

### 2. Type-Driven Dispatch for Extensions
When the codegen sees `x.method()`, it checks `var_types` to find the type of `x`, then looks up whether `method` exists in `ext_methods` for that type. If found, it rewrites to `_ext_Type_method(x)`. Otherwise, it emits a normal JS method call.

### 3. `@external(js, ...)` Attribute System
Types can bridge to native JS APIs via attributes:
- `@external(js, property, "length")` → `__self.length`
- `@external(js, method, "push")` → `__self.push(args)`
- `@external(js, static, "Math", "floor")` → `Math.floor(args)`
- `@external(constructor)` → `new TypeName(args)`

All external handling lives in `external.rs` via `emit_external_body()`.

### 4. Match Optimization
The codegen inspects match arms: if all patterns are simple literals or enum variants (no guards, no struct patterns), it emits a `switch` statement. Otherwise, it falls back to an if-else chain. See `can_emit_switch()` in `match_logic.rs`.

### 5. Post-Processing
After codegen, `postprocess::add_runtime_imports()` scans the generated JS for `__print(`, `__range(`, and `_ext_` markers, prepending the appropriate `import` statements. This runs in the CLI, not inside the emitter.

---

## How to Add a New Language Feature

1. **AST**: Add the new node to `ExprKind` or `StmtKind` in `auwla-ast`
2. **Parser**: Parse it into the new AST node in `auwla-parser`
3. **Typechecker**: Add type rules in `auwla-typechecker`
4. **Codegen**: Add a match arm in `emit_expr()` (expr.rs) or `emit_stmt_inner()` (stmt.rs)
5. **Test**: Add a `.aw` test file in `tests/`, run `cargo run -- tests`

## How to Add a New Backend (e.g., WASM, TypeScript)

The current architecture is JS-specific but modular enough to extract:
1. Create a new crate (e.g., `auwla-codegen-wasm`)
2. Reuse `CodeWriter` and `postprocess` patterns
3. Implement a new emitter struct with its own `emit_expr` / `emit_stmt` methods
4. The AST, typechecker, and parser require **zero changes**

## How to Add Optimizations

Optimization passes should operate on the AST **between** typechecking and codegen:

```
Typechecker → [Optimization Passes] → Codegen
```

Examples:
- **Constant folding**: Walk the AST, replace `Binary { 2 + 3 }` with `NumberLit(5)`
- **Dead code elimination**: Remove unreachable branches using type info
- **Inlining**: Replace simple function calls with their bodies

The `JsEmitter` already receives the full typed AST, so optimization passes can be pure `AST → AST` transformations that slot in without touching the emitter.

---

## Testing

```bash
# Build everything
cargo build

# Run all tests (single-file tests + module project)
cargo run -- tests

# Run a single file
cargo run -- tests/06_match.aw

# Debug: dump the AST
AUWLA_DEBUG=1 cargo run -- tests/06_match.aw
```

Output goes to `tests/output/*.js`. The test suite verifies that typechecking passes and JS is generated for all 16+ test files.
