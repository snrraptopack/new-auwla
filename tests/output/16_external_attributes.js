import { __print, __range } from './__util.js';
import * as __auwla from './__runtime.js';
function main() {
  const arr = [1, 2, 3];
  __print(`the max value : ${__auwla._ext_array_number_max(arr)}`);
  __print(`Length: ${__auwla._ext_array_len(arr)}`);
  __auwla._ext_array_push_val(arr, 4);
  __print(`New length: ${__auwla._ext_array_len(arr)}`);
  const check = __auwla._ext_array_is_arr(arr);
  __print(`Is array: ${check}`);
  return (() => {
    const __match_0 = __auwla._ext_array_last(arr);
    if (__match_0.ok) {
      const v = __match_0.value;
      return __print(`Last: ${v}`);
    }
    else if (!__match_0.ok) {
      return __print("Empty");
    }
})();
}
main();
__print("hello".repeat(10));
const nums = __range(1, 100, true);
for (let i = __auwla._ext_array_low(nums); i <= __auwla._ext_array_high(nums); i++) {
  __print(`the current number is : ${i}`);
}
