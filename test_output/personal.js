import { __print } from './__util.js';
function divide(f1, f2) {
  if ((f2 === 0)) {
    ({ ok: false, value: "can't divide by 0" });
  } else {
    ({ ok: true, value: (f1 / f2) });
  }
}
__print(divide(10, 0));
__print(divide(10, 3));
