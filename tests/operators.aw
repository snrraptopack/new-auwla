fn test_compound() {
    print("--- Test: Compound Assignment ---");
    var x = 10;
    x += 5;
    print("x += 5: ", x); // 15
    x -= 3;
    print("x -= 3: ", x); // 12
    x *= 2;
    print("x *= 2: ", x); // 24
    x /= 4;
    print("x /= 4: ", x); // 6
    x %= 4;
    print("x %= 4: ", x); // 2

    var s = "Hello";
    s += " World";
    print("s += ' World': ", s); // Hello World
}
type result<T, E> = T?E;
type optional<T> = T?;

fn test_coalesce() {
    print("--- Test: Nullish Coalescing ---");
    let opt_some: optional<number> = some(42);
    let opt_none: optional<number> = none;

    let val1 = opt_some ?? 0;
    let val2 = opt_none ?? 100;

    print("some(42) ?? 0: ", val1); // 42
    print("none() ?? 100: ", val2); // 100

    let res_ok: result<string, string> = some("Success");
    let res_err: result<string, string> = none("Failure");

    let val3 = res_ok ?? "Default";
    let val4 = res_err ?? "Fallback";


    print("result::ok('Success') ?? 'Default': ", val3); // Success
    print("result::err('Failure') ?? 'Fallback': ", val4); // Fallback
}

fn test_strict_bool() {
    print("--- Test: Strict Boolean ---");
    let t = true;
    let f = false;
    if t && !f {
        print("t && !f is true");
    }
    if t || f {
        print("t || f is true");
    }

    // The following should FAIL typechecking if uncommented:
    // let invalid = 1 && 2;
    // let invalid2 = "hi" || "there";
}


    test_compound();
    test_coalesce();
    test_strict_bool();
