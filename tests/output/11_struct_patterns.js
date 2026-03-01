import { __print } from './__util.js';
const alice = { name: "Alice", age: 30, role: "admin" };
const bob = { name: "Bob", age: 25, role: "user" };
const charlie = { name: "Charlie", age: 16, role: "user" };
const { name, age } = alice;
__print(("Extracted from Alice: " + name));
function greet(u) {
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
}
greet(alice);
greet(bob);
greet(charlie);
