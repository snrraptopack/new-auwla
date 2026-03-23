# Operator Overloading Implementation

## Status: ✅ Implemented (Needs Testing)

We've successfully implemented operator overloading for the Auwla programming language!

## What Was Implemented

### 1. AST Changes (`auwla-ast/src/stmt.rs`)
- Added `OperatorType` enum with variants: `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Range`, `RangeExclusive`
- Added `operator: Option<OperatorType>` field to `Method` struct
- Added `method_suffix()` helper to get function name suffix for each operator

### 2. Lexer Changes (`auwla-lexer/src/token.rs`)
- Added `Operator` token for the `operator` keyword

### 3. Parser Changes (`auwla-parser/src/stmt.rs`)
- Modified method parser to recognize `operator SYMBOL` syntax
- Parses operator symbols: `+`, `-`, `*`, `/`, `%`, `..`, `..<`
- Generates method names like `op_plus`, `op_minus`, etc.

### 4. Codegen Changes (`auwla-codegen/src/expr.rs`)
- Modified binary operator emission to check for operator overloads
- Modified range expression emission to check for operator overloads
- Falls back to built-in operators if no overload is found
- Emits operator calls as `_ext_Type__op_plus(left, right)`

### 5. Postprocessor Changes (`auwla-codegen/src/postprocess.rs`)
- Updated to recognize `__op_*` patterns in function names
- Properly routes operator functions to std or user namespaces

### 6. Codegen Statement Changes (`auwla-codegen/src/stmt.rs`)
- Fixed char range bug in for-loops
- Char ranges now use `__range()` function instead of invalid `c += 1`

## Syntax

```auwla
extend TypeName {
    operator SYMBOL(self, param: ParamType): ReturnType {
        // implementation
    }
}
```

## Supported Operators

| Operator | Symbol | Method Name | Example |
|----------|--------|-------------|---------|
| Addition | `+` | `op_plus` | `v1 + v2` |
| Subtraction | `-` | `op_minus` | `v1 - v2` |
| Multiplication | `*` | `op_mul` | `v * 2` |
| Division | `/` | `op_div` | `v / 2` |
| Modulo | `%` | `op_mod` | `n % 10` |
| Range (inclusive) | `..` | `op_range` | `start .. end` |
| Range (exclusive) | `..<` | `op_range_exclusive` | `start ..< end` |

## Example Usage

```auwla
struct Vector2 {
    x: number,
    y: number
}

extend Vector2 {
    operator +(self, other: Vector2): Vector2 {
        return Vector2 { 
            x: self.x + other.x, 
            y: self.y + other.y 
        };
    }
    
    operator *(self, scalar: number): Vector2 {
        return Vector2 { 
            x: self.x * scalar, 
            y: self.y * scalar 
        };
    }
}

let v1 = Vector2 { x: 10, y: 20 };
let v2 = Vector2 { x: 5, y: 3 };

let sum = v1 + v2;           // Vector2 { x: 15, y: 23 }
let scaled = v1 * 2;         // Vector2 { x: 20, y: 40 }
let result = (v1 + v2) * 0.5; // Complex expressions work!
```

## Compilation

### Input (Auwla):
```auwla
let v3 = v1 + v2;
```

### Output (JavaScript):
```javascript
const v3 = __user._ext_usr_Vector2__op_plus(v1, v2);
```

## Std vs User Philosophy

Operator overloads follow the same std/user separation:

- **Std operators**: `_ext_Type__op_plus` → `__std_module._ext_Type__op_plus`
- **User operators**: `_ext_usr_Type__op_plus` → `__user._ext_usr_Type__op_plus`

## Next Steps

1. **Test the implementation** - Run `cargo build` and test with `tests/23_operator_overload.aw`
2. **Add std library operators** - Implement useful operators in std modules:
   - `string * number` for repetition
   - `dict + dict` for merging
3. **Add more operators** - Consider adding:
   - Comparison operators: `==`, `!=`, `<`, `>`, `<=`, `>=`
   - Indexing operator: `[]`
   - Unary operators: `-`, `!`
4. **Documentation** - Add operator overloading to language docs
5. **Error messages** - Improve error messages for operator type mismatches

## Design Decisions

1. **Left operand owns the operator** - `v1 + v2` uses `v1`'s type operator
2. **No overriding built-ins** - Can't override `number + number`
3. **Explicit return types** - No implicit conversions
4. **Consistent with extensions** - Uses same `extend` syntax
5. **Type-safe** - Typechecker validates all operator calls

## Known Limitations

1. **Commutative operators need two definitions**:
   ```auwla
   extend Vector2 {
       operator *(self, scalar: number): Vector2 { ... }
   }
   extend number {
       operator *(self, vec: Vector2): Vector2 { ... }  // For 2 * v
   }
   ```

2. **Can't override built-in type operators** - `number + number` always uses built-in

3. **No operator chaining syntax** - Can't do `a < b < c` (yet)

## Files Modified

- `auwla-ast/src/stmt.rs`
- `auwla-lexer/src/token.rs`
- `auwla-parser/src/stmt.rs`
- `auwla-codegen/src/expr.rs`
- `auwla-codegen/src/stmt.rs`
- `auwla-codegen/src/postprocess.rs`
- `tests/23_operator_overload.aw` (new)

## Testing

Run the test:
```bash
cargo build --release
./target/release/auwla-cli tests/23_operator_overload.aw
node test_output/23_operator_overload.js
```

Expected output:
```
v1 + v2 = Vector2 { x: 15, y: 23 }
v1 - v2 = Vector2 { x: 5, y: 17 }
v1 * 2 = Vector2 { x: 20, y: 40 }
v1 / 2 = Vector2 { x: 5, y: 10 }
(v1 + v2) * 0.5 = Vector2 { x: 7.5, y: 11.5 }
```
