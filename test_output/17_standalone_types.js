import { __print } from './__util.js';
function main() {
  const x = Math.floor(3.7);
  __print(`floor(3.7) = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(x)}`);
  const y = Math.ceil(3.2);
  __print(`ceil(3.2) = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(y)}`);
  const r = Math.random();
  __print(`random = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(r)}`);
  const pi = Math.PI;
  __print(`PI = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(pi)}`);
  console.log("hello from Console::log");
}
main();
