import { __print } from './__util.js';
import * as __auwla from './__runtime.js';
for (let i = 1; i <= 5; i++) {
  const d = i.double();
  const s = i.square();
  __print(`${i} → double: ${d}, square: ${s}`);
}
let count = 1;
while ((count < 6)) {
  __print(`${count} tripled = ${__auwla._ext_number_triple(count)}`);
  count = (count + 1);
}
const result = __auwla._ext_number_double_then_square(3);
__print(`3.double_then_square() = ${result}`);
const nums = [10, 20, 30];
for (const n of nums) {
  const processed = n.double().add(5);
  __print(`${n}.double().add(5) = ${processed}`);
}
