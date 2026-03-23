import { __print } from './__util.js';
import * as __std_number from './std/number.js';
import * as __user from './__user_ext.js';
for (let i = 1; i <= 5; i += 1) {
  const d = __std_number._ext_number__double(i);
  const s = __std_number._ext_number__square(i);
  __print(`${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(i)} → double: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(d)}, square: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(s)}`);
}
let count = 1;
while ((count < 6)) {
  __print(`${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(count)} tripled = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(__std_number._ext_number__triple(count))}`);
  count = (count + 1);
}
const result = __user._ext_usr_number__double_then_square(3);
__print(`3.double_then_square() = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(result)}`);
const nums = [10, 20, 30];
for (const n of nums) {
  const processed = __std_number._ext_number__add(__std_number._ext_number__double(n), 5);
  __print(`${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(n)}.double().add(5) = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(processed)}`);
}
