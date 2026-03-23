import { __print } from './__util.js';
function test_compound() {
  __print("--- Test: Compound Assignment ---");
  let x = 10;
  x += 5;
  __print("x += 5: ", x);
  x -= 3;
  __print("x -= 3: ", x);
  x *= 2;
  __print("x *= 2: ", x);
  x /= 4;
  __print("x /= 4: ", x);
  x %= 4;
  __print("x %= 4: ", x);
  let s = "Hello";
  s += " World";
  __print("s += ' World': ", s);
}
function test_coalesce() {
  __print("--- Test: Nullish Coalescing ---");
  const opt_some = ({ ok: true, value: 42 });
  const opt_none = ({ ok: false });
  const val1 = ((_o) => _o.ok ? _o.value : (0))(opt_some);
  const val2 = ((_o) => _o.ok ? _o.value : (100))(opt_none);
  __print("some(42) ?? 0: ", val1);
  __print("none() ?? 100: ", val2);
  const res_ok = ({ ok: true, value: "Success" });
  const res_err = ({ ok: false, value: "Failure" });
  const val3 = ((_o) => _o.ok ? _o.value : ("Default"))(res_ok);
  const val4 = ((_o) => _o.ok ? _o.value : ("Fallback"))(res_err);
  __print("result::ok('Success') ?? 'Default': ", val3);
  __print("result::err('Failure') ?? 'Fallback': ", val4);
}
function test_strict_bool() {
  __print("--- Test: Strict Boolean ---");
  const t = true;
  const f = false;
  if ((t && !f)) {
    __print("t && !f is true");
  }
  if ((t || f)) {
    __print("t || f is true");
  }
}
test_compound();
test_coalesce();
test_strict_bool();
