import { __print } from './__util.js';
import * as __user from './__user_ext.js';
const base_arr = [2, 3];
const spread_arr = [1, ...base_arr, 4];
__print("Spread Array: ");
for (const x of spread_arr) {
  __print(x);
}
const base_dict = { "a": 2, "c": 3 };
const spread_dict = { "a": 1, ...base_dict, "d": 4 };
__print("Spread Dict keys (approx):");
function sum_all(...nums) {
  let total = 0;
  for (const n of nums) {
    total += n;
  }
  return total;
}
const s = sum_all(1, 2, 3, 4, 5);
__print("Sum All (1..5):");
__print(s);
__print("Step loop (0 to 10 step 2):");
for (let i = 0; i <= 11; i += 2) {
  __print(i);
}
const val = 10;
const val2 = __user._ext_usr_number__multi_add(val, 1, 1, 1);
__print("Method multi_add (10 + 1 + 1 + 1):");
__print(val2);
