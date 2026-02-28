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

fn checked_opt(n: number): string? {
    if n > 10 {
        return some("Opt Value is {n}");
    } else {
        return none;
    }
}

fn test_opt_auto(n: number): string? {
    let val = checked_opt(n)?;
    return some("Opt Success: {val}");
}

print("Testing Try Operator");
print(test_auto(15));
print(test_auto(5));
print(test_override(5));

print("Testing Optional Try");
print(test_opt_auto(15));
print(test_opt_auto(5));
