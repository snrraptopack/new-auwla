function checked(n) {
  if ((n > 10)) {
    return ({ ok: true, value: `Value is ${n}` });
  } else {
    return ({ ok: false, value: "Original Error" });
  }
}
function test_override(n) {
  const __match_0 = checked(n);
  if (!__match_0.ok) return { ok: false, value: "Fallback error" };
  const val = __match_0.value;
  return ({ ok: true, value: `Override success: ${val}` });
}
function test_auto(n) {
  const __match_1 = checked(n);
  if (!__match_1.ok) return __match_1;
  const val = __match_1.value;
  return ({ ok: true, value: `Auto success: ${val}` });
}
console.log("--- Test Override (Success) ---");
const __match_2 = test_override(20);
if (__match_2.ok) {
  const v = __match_2.value;
  console.log(`Success: ${v}`);
} else {
  const e = __match_2.value;
  console.log(`Error: ${e}`);
}
console.log("--- Test Override (Fail) ---");
const __match_3 = test_override(5);
if (__match_3.ok) {
  const v = __match_3.value;
  console.log(`Success: ${v}`);
} else {
  const e = __match_3.value;
  console.log(`Error: ${e}`);
}
console.log("--- Test Auto (Success) ---");
const __match_4 = test_auto(20);
if (__match_4.ok) {
  const v = __match_4.value;
  console.log(`Success: ${v}`);
} else {
  const e = __match_4.value;
  console.log(`Error: ${e}`);
}
console.log("--- Test Auto (Fail) ---");
const __match_5 = test_auto(5);
if (__match_5.ok) {
  const v = __match_5.value;
  console.log(`Success: ${v}`);
} else {
  const e = __match_5.value;
  console.log(`Error: ${e}`);
}
