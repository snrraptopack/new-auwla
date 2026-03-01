import { __print } from './__util.js';
import * as __auwla from './__runtime.js';
function main() {
  const arr = [1, 2, 3];
  __print(`Length: ${__auwla.__ext_array_len(arr)}`);
  __auwla.__ext_array_push_val(arr, 4);
  __print(`New length: ${__auwla.__ext_array_len(arr)}`);
  const check = __auwla.__ext_array_is_arr(arr);
  __print(`Is array: ${check}`);
  return (() => {
    const __match_0 = __auwla.__ext_array_last(arr);
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
