// 12_closures.aw
// Testing anonymous functions and arrow functions

// 1. Basic Arrow Function
let double = (x: number): number => x * 2;
print("Double 21: {double(21)}");

// 2. Closure with Block Body
let multiply = (x: number, y: number): number => {
    let result = x * y;
    return result;
};
print("Multiply 6 * 7: {multiply(6, 7)}");

// 3. Passing Closure as a Callback
fn apply_twice(val: number, f: (number) => number): number {
    return f(f(val));
}

let result = apply_twice(5, (x: number) => x + 10);
print("Apply twice (5 + 10 + 10): {result}");

// 4. Inferred parameter types (Note: current implementation might require annotations, let's test)
fn run_callback(f: () => void) {
    f();
}

run_callback(() => {
    print("Callback executed successfully!");
});

// 5. Closure capturing outside variable (Lexical scoping)
let greeting = "Hello";
let greet = (name: string) => {
    print("{greeting}, {name}!");
};
greet("Auwla");
