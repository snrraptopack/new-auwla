let num: number = 42;
var a = "hello";
a = "world";

fn checked(n: number): string?string {
    if n > 10 {
        return some("Too big!");
    } else {
        return none("Okay!");
    }
}

// print takes any expression
print("Number:", num);
print("String:", a);
print("Bool:", true);
print("Expr:", 10 + 20 * 2);

// Match as expression
let result = checked(num);
let msg = match result {
    some(val) => val
    none(err) => err
};
print("Match result:", msg);

// Match with block arms
var result2 = checked(5);
let msg2 = match result2 {
    some(val) => {
        let upper = val;
        upper
    }
    none(err) => err
};
print("Match block:", msg2);

// Standalone match
var result3 = checked(3);
match result3 {
    some(val) => {
        print("Got value:", val);
    }
    none(err) => {
        print("Got error:", err);
    }
}
