import { __print, __range } from './__util.js';
for (let x = 1; x <= 5; x++) {
  __print(`Inclusive: ${x}`);
}
for (let y = 1; y < 5; y++) {
  __print(`Exclusive: ${y}`);
}
for (let c = "a"; c <= "d"; c++) {
  __print(`Char: ${c}`);
}
const a = __range("a", "z", true);
__print(`Array from range: ${a}`);
