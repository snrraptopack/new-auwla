// 05_try_operator.aw
fn checked(n: number): string?string {
    if n > 10 {
        return some("Value is {n}");
    } else {
        return none("Error!");
    }
}

fn test_auto(n: number): string?string {
    let val = checked(n)?;
    return some("Success: {val}");
}

fn test_override(n: number): string?string {
    let val = checked(n)?("Override Error");
    return some("Success: {val}");
}

print("Testing Try Operator");
