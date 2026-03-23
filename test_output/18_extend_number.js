import { __print } from './__util.js';
import * as __std_number from './std/number.js';
import * as __user from './__user_ext.js';
for (let i = 1; i <= 5; i += 1) {
  const d = __std_number._ext_number__double(i);
  const s = __std_number._ext_number__square(i);
  __print(`${i} → double: ${d}, square: ${s}`);
}
let count = 1;
while ((count < 6)) {
  __print(`${count} tripled = ${__std_number._ext_number__triple(count)}`);
  count = (count + 1);
}
const result = __user._ext_usr_number__double_then_square(3);
__print(`3.double_then_square() = ${result}`);
const nums = [10, 20, 30];
for (const n of nums) {
  const processed = __std_number._ext_number__add(__std_number._ext_number__double(n), 5);
  __print(`${n}.double().add(5) = ${processed}`);
}
