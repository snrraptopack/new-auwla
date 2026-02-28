const double = (x) => (x * 2);
console.log(`Double 21: ${double(21)}`);
const multiply = (x, y) => {
    const result = (x * y);
    return result;
};
console.log(`Multiply 6 * 7: ${multiply(6, 7)}`);
function apply_twice(val, f) {
  return f(f(val));
}
const result = apply_twice(5, (x) => (x + 10));
console.log(`Apply twice (5 + 10 + 10): ${result}`);
function run_callback(f) {
  f();
}
run_callback(() => {
    console.log("Callback executed successfully!");
});
const greeting = "Hello";
const greet = (name) => {
    console.log(`${greeting}, ${name}!`);
};
greet("Auwla");
