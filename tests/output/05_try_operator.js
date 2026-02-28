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
console.log("Testing Try Operator");
