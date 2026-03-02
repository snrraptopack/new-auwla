import { __print } from './__util.js';
function main() {
  const x = Math.floor(3.7);
  __print(`floor(3.7) = ${x}`);
  const y = Math.ceil(3.2);
  __print(`ceil(3.2) = ${y}`);
  const r = Math.random();
  __print(`random = ${r}`);
  const pi = Math.PI;
  __print(`PI = ${pi}`);
  Console.log("hello from Console::log");
}
main();
