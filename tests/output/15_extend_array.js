import { __print } from './__util.js';
import * as __auwla from './__runtime.js';
const names = ["Alice", "Bob", "Charlie"];
const last_name = __auwla._ext_array_last(names);
const __match_0 = last_name;
if (__match_0.ok) {
  const name = __match_0.value;
  __print(`Last name: ${name}`);
}
else if (!__match_0.ok) {
  __print("No names");
}
const numbers = [1, 2, 3, 4, 5];
const last_num = __auwla._ext_array_last(numbers);
const __match_1 = last_num;
if (__match_1.ok) {
  const n = __match_1.value;
  __print(`Last number: ${n}`);
}
else if (!__match_1.ok) {
  __print("No numbers");
}
