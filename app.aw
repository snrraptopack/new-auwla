fn checked(n: number): string?string {
    if n > 10 {
        return some("Value is {n}");
    } else {
        return none("Original Error");
    }
}

// Uses explicit override `?("Fallback error")`
fn test_override(n: number): string?string {
    let val = checked(n)?("Fallback error");
    return some("Override success: {val}");
}

// Uses automatic propagation `?`
fn test_auto(n: number): string?string {
    let val = checked(n)?;
    return some("Auto success: {val}");
}

print("--- Test Override (Success) ---");
match test_override(20) {
    some(v) => print("Success: {v}")
    none(e) => print("Error: {e}")
}

print("--- Test Override (Fail) ---");
match test_override(5) {
    some(v) => print("Success: {v}")
    none(e) => print("Error: {e}")
}

print("--- Test Auto (Success) ---");
match test_auto(20) {
    some(v) => print("Success: {v}")
    none(e) => print("Error: {e}")
}

print("--- Test Auto (Fail) ---");
match test_auto(5) {
    some(v) => print("Success: {v}")
    none(e) => print("Error: {e}")
}
