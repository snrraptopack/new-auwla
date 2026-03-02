import { __print } from '../../output/__util.js';
import * as __auwla from '../../output/__runtime.js';
function main() {
  const arr = [10, 20, 30];
  __print(`Sum from extension: ${__auwla._ext_array_number_sum(arr)}`);
  __print(arr.length);
}
main();
