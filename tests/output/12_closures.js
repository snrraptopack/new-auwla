function __print(...args) {
  const format = (val, top = false) => {
    if (val && typeof val === 'object' && 'ok' in val) {
      if (val.ok) return `some(${format(val.value)})`;
      if ('value' in val) return `none(${format(val.value)})`;
      return 'none';
    }
    if (Array.isArray(val)) return `[${val.map(v => format(v)).join(', ')}]`;
    if (typeof val === 'string' && !top) return `"${val}"`;
    if (typeof val === 'object' && val !== null) {
      const props = Object.entries(val).map(([k, v]) => `${k}: ${format(v)}`).join(', ');
      return `{ ${props} }`;
    }
    return val;
  };
  console.log(...args.map(a => format(a, true)));
}

const double = (x) => (x * 2);
__print(`Double 21: ${double(21)}`);
const multiply = (x, y) => {
    const result = (x * y);
    return result;
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
});
const greeting = "Hello";
const greet = (name) => {
    __print(`${greeting}, ${name}!`);
};
greet("Auwla");
