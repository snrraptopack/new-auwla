import { __print, __range } from './__util.js';
for (let x = 1; x <= 5; x += 1) {
  __print(`Inclusive: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(x)}`);
}
for (let y = 1; y < 5; y += 1) {
  __print(`Exclusive: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(y)}`);
}
for (const c of __range("a", "d", true)) {
  __print(`Char: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(c)}`);
}
const a = __range("a", "z", true);
__print(`Array from range: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(a)}`);
