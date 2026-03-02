import { __print } from './__util.js';
import * as __auwla from './__runtime.js';
function main() {
  const f = __auwla._ext_Math_floor(3.9);
  __print("Math.floor(3.9) =", f);
  const p = __auwla._ext_Math_pi();
  __print("Math.PI =", p);
  const d = new Date();
  __print("New Date getTime =", d.get_time());
  const n = __auwla._ext_Date_now();
  __print("Date.now() =", n);
}
main();
