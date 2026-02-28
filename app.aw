fn checked(n: number): string?string {
    if n > 10 {
        return some("Value is {n}");
    } else {
        return none("Error!");
    }
}

fn test_try(n: number): string?string {
    // Try operator: unwraps and assigns to 'val' OR returns early with "Fallback" error
    let val = checked(n)?("Fallback error from try");
    print("Inside test_try, unwrapped: {val}");
    return some("Everything is fine: {val}");
}

print("--- Test 1 (Success) ---");
let r1 = test_try(20);
match r1 {
    some(v) => print("Success output: {v}")
    none(e) => print("Error output: {e}")
}

print("--- Test 2 (Fail) ---");
let r2 = test_try(5);
match r2 {
    some(v) => print("Success output: {v}")
    none(e) => print("Error output: {e}")
}

// ── Other features still working ──
print("--- Ranges & Arrays ---");
for x in 1..3 {
    print("Loop x: {x}");
}
let arr = ["A", "B"];
print("Array: {arr}, Fragment: {arr[0]}");
