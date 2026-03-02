import { __print } from './__util.js';
const double = (x) => (x * 2);
__print(`Double 21: ${double(21)}`);
const multiply = (x, y) => {
    const result = (x * y);
    return result;
  return undefined;
};
__print(`Multiply 6 * 7: ${multiply(6, 7)}`);
function apply_twice(val, f) {
  return f(f(val));
}
const result = apply_twice(5, (x) => (x + 10));
__print(`Apply twice (5 + 10 + 10): ${result}`);
function run_callback(f) {
  f();
}
run_callback(() => {
    __print("Callback executed successfully!");
  return undefined;
});
const greeting = "Hello";
const greet = (name) => {
    __print(`${greeting}, ${name}!`);
  return undefined;
};
greet("Auwla");
