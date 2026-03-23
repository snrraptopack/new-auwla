import { __print } from '../../../test_output/__util.js';
import * as __std_array from '../../../test_output/std/array.js';
function main() {
  const arr = [10, 20, 30];
  __print(`Sum from extension: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(__std_array._ext_array__sum(arr))}`);
  __print(arr.length);
}
main();
