let msg: string?string = some("Hello Worlds!");
let num: number = 42;

// should error — num is `let`, not `var`
num = 30;

// var infers `string` from the initializer
var a = "hello";
a = "world";   // ✓ ok — still a string

// uncommenting the next line would produce:
// → Strict Type mismatch: Expected 'string', found 'number'
// a = 10;

fn greet(name: string): string {
    return name;
}

fn checked(a: number): string?string {
    if a > 10 {
        return some("Too big!");
    } else {
        return none("Okay!");
    }
}
