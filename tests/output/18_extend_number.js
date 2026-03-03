import { __print } from './__util.js';
import * as __std_number from './std/number.js';
for (let i = 1; i <= 5; i++) {
  const d = _ext_number_double(i);
  const s = _ext_number_square(i);
  __print(`${i} → double: ${d}, square: ${s}`);
}
let count = 1;
while ((count < 6)) {
  __print(`${count} tripled = ${_ext_number_triple(count)}`);
  count = (count + 1);
}
const result = _ext_number_double_then_square(3);
__print(`3.double_then_square() = ${result}`);
const nums = [10, 20, 30];
for (const n of nums) {
  const processed = _ext_number_add(_ext_number_double(n), 5);
  __print(`${n}.double().add(5) = ${processed}`);
}
