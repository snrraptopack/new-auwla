import { __print, __range } from './__util.js';
import * as __std_array from './std/array.js';
function main() {
  const arr = [1, 2, 3];
  __print(`the max value : ${_ext_array_number_max(arr)}`);
  __print(`Length: ${arr.length}`);
  arr.push(4);
  __print(`New length: ${arr.length}`);
  const check = Array.isArray(arr);
  __print(`Is array: ${check}`);
  const __match_0 = _ext_array_last(arr);
  if (__match_0.ok) {
    const v = __match_0.value;
    __print(`Last: ${v}`);
  }
  else if (!__match_0.ok) {
    __print("Empty");
  }
}
const aaa = ((_r = "ama".at(1)) != null ? { ok: true, value: _r } : { ok: false });
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
for (let i = _ext_array_low(nums); i <= _ext_array_high(nums); i++) {
  __print(`the current number is : ${i}`);
}
