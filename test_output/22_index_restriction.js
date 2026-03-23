import { __print } from './__util.js';
import * as __std_array from './std/array.js';
import * as __std_dict from './std/dict.js';
import * as __std_optional from './std/optional.js';
function main() {
  const arr = [1, 2, 3];
  const val = __std_optional._ext_optional__val_or(__std_array._ext_array__get(arr, 0), 0);
  __print(`Array get(0).val_or(0): ${val}`);
  const missing = __std_optional._ext_optional__val_or(__std_array._ext_array__get(arr, 10), -1);
  __print(`Array get(10).val_or(-1): ${missing}`);
  const d = { "a": 1, "b": 2 };
  const dv = __std_optional._ext_optional__val_or(__std_dict._ext_dict__get(d, "a"), 0);
  __print(`Dict get(\"a\").val_or(0): ${dv}`);
  const d_missing = __std_optional._ext_optional__val_or(__std_dict._ext_dict__get(d, "z"), 404);
  __print(`Dict get(\"z\").val_or(404): ${d_missing}`);
}
main();
