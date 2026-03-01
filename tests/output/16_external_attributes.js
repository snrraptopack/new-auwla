import { __print } from './__util.js';
import * as __auwla from './__runtime.js';
function main() {
  const arr = [1, 2, 3];
  __print(`the max value : ${__auwla.__ext_array_number__max(arr)}`);
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
__print("hello".repeat(10));
const nums = ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})(1, 100);
for (const i of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})(1, 100)) {
  __print(`the current number is : ${i}`);
}
