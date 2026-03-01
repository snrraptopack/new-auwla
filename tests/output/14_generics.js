import { __print } from './__util.js';
function identity(x) {
  return x;
}
function main() {
  const x = identity(42);
  const y = identity("hello");
  const z = identity("forced");
  let again = identity(10);
  again = 10;
  const list = ["a", "b"];
  __print(x);
  __print(y);
  __print(z);
  __print(list);
}
