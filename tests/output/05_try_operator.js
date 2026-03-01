import { __print } from './__util.js';
function checked(n) {
  if ((n > 10)) {
    return ({ ok: true, value: `Value is ${n}` });
  } else {
    return ({ ok: false, value: "Error!" });
  }
}
function test_auto(n) {
  const __match_0 = checked(n);
  if (!__match_0.ok) return __match_0;
  const val = __match_0.value;
  return ({ ok: true, value: `Success: ${val}` });
}
function test_override(n) {
  const __match_1 = checked(n);
  if (!__match_1.ok) return { ok: false, value: "Override Error" };
  const val = __match_1.value;
  return ({ ok: true, value: `Success: ${val}` });
}
function checked_opt(n) {
  if ((n > 10)) {
    return ({ ok: true, value: `Opt Value is ${n}` });
  } else {
    return ({ ok: false });
  }
}
function test_opt_auto(n) {
  const __match_2 = checked_opt(n);
  if (!__match_2.ok) return __match_2;
  const val = __match_2.value;
  return ({ ok: true, value: `Opt Success: ${val}` });
}
__print("Testing Try Operator");
__print(test_auto(15));
__print(test_auto(5));
__print(test_override(5));
__print("Testing Optional Try");
__print(test_opt_auto(15));
__print(test_opt_auto(5));
