let num: number = 42;

// var infers `string` from the initializer
var a = "hello";
a = "world";

fn checked(n: number): string?string {
    if n > 10 {
        return some("Too big!");
    } else {
        return none("Okay!");
    }
}

// Match as expression — both arms must yield the same type
let result = checked(num);
let msg = match result {
    some(val) => val
    none(err) => err
};

// Match with block arms
var result2 = checked(5);
let msg2 = match result2 {
    some(val) => {
        let upper = val;
        upper
    }
    none(err) => err
};

// Standalone match (no assignment, no semicolon)
var result3 = checked(3);
match result3 {
    some(val) => {
        let x = val;
    }
    none(err) => {
        let y = err;
    }
}
