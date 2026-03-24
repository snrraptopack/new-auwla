import { __print } from './__util.js';
const alice = { name: "Alice", age: 30, role: { $variant: "Admin" } };
const bob = { name: "Bob", age: 25, role: { $variant: "User" } };
const charlie = { name: "Charlie", age: 28, role: { $variant: "Moderator" } };
function greet(u) {
  const __match_0 = u;
  if (((__match_0.role.$variant === "Admin" || __match_0.role.$variant === "Moderator") && __match_0.name !== undefined)) {
    const name = __match_0.name;
    __print(`Welcome back, Staff ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)}`);
  }
  else if ((__match_0.role.$variant === "User" && __match_0.age !== undefined) && (() => {
    const age = __match_0.age;
    return (age < 18);
  })()) {
    const age = __match_0.age;
    __print("You are not old enough!");
  }
  else if ((__match_0.name !== undefined && __match_0.age !== undefined)) {
    const name = __match_0.name;
    const age = __match_0.age;
    __print(`Welcome, ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)} (${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(age)})`);
  }
}
greet(alice);
greet(bob);
greet(charlie);
