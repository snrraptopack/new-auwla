import { __print, __range } from './__util.js';
for (let x = 1; x <= 5; x += 1) {
  __print(`Inclusive: ${x}`);
}
for (let y = 1; y < 5; y += 1) {
  __print(`Exclusive: ${y}`);
}
for (const c of __range("a", "d", true)) {
  __print(`Char: ${c}`);
}
const a = __range("a", "z", true);
__print(`Array from range: ${a}`);
