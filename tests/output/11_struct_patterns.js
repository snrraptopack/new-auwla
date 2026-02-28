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

const alice = { name: "Alice", age: 30, role: "admin" };
const bob = { name: "Bob", age: 25, role: "user" };
const charlie = { name: "Charlie", age: 16, role: "user" };
const { name, age } = alice;
__print(("Extracted from Alice: " + name));
function greet(u) {
  return (() => {
    const __match_0 = u;
    if ((__match_0.role === "admin" && __match_0.name !== undefined)) {
      const name = __match_0.name;
      return __print(`Welcome back, Admin ${name}`);
    }
    else if ((__match_0.role === "user" && __match_0.age !== undefined) && (() => {
      const age = __match_0.age;
      return (age < 18);
    })()) {
      const age = __match_0.age;
      return __print("You are not old enough!");
    }
    else if ((__match_0.name !== undefined && __match_0.age !== undefined)) {
      const name = __match_0.name;
      const age = __match_0.age;
      return __print(`Welcome, ${name} (${age})`);
    }
})();
}
greet(alice);
greet(bob);
greet(charlie);
