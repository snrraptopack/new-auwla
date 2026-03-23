import { __print, __range } from './__util.js';
import * as __std_array from './std/array.js';
function main() {
  const arr = [1, 2, 3];
  __print(`the max value : ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(__std_array._ext_array__max(arr))}`);
  __print(`Length: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(arr.length)}`);
  arr.push(4);
  __print(`New length: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(arr.length)}`);
  const check = Array.isArray(arr);
  __print(`Is array: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(check)}`);
  const __match_0 = __std_array._ext_array__last(arr);
  if (__match_0.ok) {
    const v = __match_0.value;
    __print(`Last: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(v)}`);
  }
  else if (!__match_0.ok) {
    __print("Empty");
  }
}
const aaa = ((_r) => _r != null ? { ok: true, value: _r } : { ok: false })("ama".at(1));
const __match_1 = aaa;
if (__match_1.ok) {
  const v = __match_1.value;
  __print(v);
}
else if (!__match_1.ok) {
  __print("none");
}
main();
__print("hello".repeat(10));
const nums = __range(1, 100, true);
for (let i = __std_array._ext_array__low(nums); i <= __std_array._ext_array__high(nums); i += 1) {
  __print(`the current number is : ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(i)}`);
}
