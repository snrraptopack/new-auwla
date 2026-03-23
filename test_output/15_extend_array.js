import { __print } from './__util.js';
import * as __std_array from './std/array.js';
const names = ["Alice", "Bob", "Charlie"];
const last_name = __std_array._ext_array__last(names);
const __match_0 = last_name;
if (__match_0.ok) {
  const name = __match_0.value;
  __print(`Last name: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)}`);
}
else if (!__match_0.ok) {
  __print("No names");
}
const numbers = [1, 2, 3, 4, 5];
const last_num = __std_array._ext_array__last(numbers);
const __match_1 = last_num;
if (__match_1.ok) {
  const n = __match_1.value;
  __print(`Last number: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(n)}`);
}
else if (!__match_1.ok) {
  __print("No numbers");
}
