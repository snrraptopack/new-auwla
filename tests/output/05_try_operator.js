function __print(...args) {
  const format = (val, top = false) => {
    if (val && typeof val === 'object' && 'ok' in val) {
      if (val.ok) return `some(${format(val.value)})`;
      if ('value' in val) return `none(${format(val.value)})`;
      return 'none';
    }
    if (Array.isArray(val)) return `[${val.map(v => format(v)).join(', ')}]`;
    if (typeof val === 'string' && !top) return `"${val}"`;
    if (typeof val === 'object' && val !== null) {
      const props = Object.entries(val).map(([k, v]) => `${k}: ${format(v)}`).join(', ');
      return `{ ${props} }`;
    }
    return val;
  };
  console.log(...args.map(a => format(a, true)));
}

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
