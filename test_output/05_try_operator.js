import { __print } from './__util.js';
function checked(n) {
  if ((n > 10)) {
    return ({ ok: true, value: `Value is ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(n)}` });
  } else {
    return ({ ok: false, value: "Error!" });
  }
}
function test_auto(n) {
  const __match_0 = checked(n);
  if (!__match_0.ok) throw new Error(__match_0.value);
  const val = __match_0.value;
  return ({ ok: true, value: `Success: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(val)}` });
}
function test_override(n) {
  const __match_1 = checked(n);
  if (!__match_1.ok) throw new Error("Override Error");
  const val = __match_1.value;
  return ({ ok: true, value: `Success: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(val)}` });
}
function checked_opt(n) {
  if ((n > 10)) {
    return ({ ok: true, value: `Opt Value is ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(n)}` });
  } else {
    return ({ ok: false });
  }
}
function test_opt_auto(n) {
  const __match_2 = checked_opt(n);
  if (!__match_2.ok) throw new Error(__match_2.value);
  const val = __match_2.value;
  return ({ ok: true, value: `Opt Success: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(val)}` });
}
__print("Testing Try Operator");
__print(test_auto(15));
__print(test_auto(5));
__print(test_override(5));
__print("Testing Optional Try");
__print(test_opt_auto(15));
__print(test_opt_auto(5));
