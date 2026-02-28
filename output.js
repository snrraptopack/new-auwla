function checked(n) {
  if ((n > 10)) {
    return ({ ok: true, value: `Value is ${n}` });
  } else {
    return ({ ok: false, value: "Error!" });
  }
}
function test_try(n) {
  const __match_0 = checked(n);
  if (!__match_0.ok) return { ok: false, value: "Fallback error from try" };
  const val = __match_0.value;
  console.log(`Inside test_try, unwrapped: ${val}`);
  return ({ ok: true, value: `Everything is fine: ${val}` });
}
console.log("--- Test 1 (Success) ---");
const r1 = test_try(20);
const __match_1 = r1;
if (__match_1.ok) {
  const v = __match_1.value;
  console.log(`Success output: ${v}`);
} else {
  const e = __match_1.value;
  console.log(`Error output: ${e}`);
}
console.log("--- Test 2 (Fail) ---");
const r2 = test_try(5);
const __match_2 = r2;
if (__match_2.ok) {
  const v = __match_2.value;
  console.log(`Success output: ${v}`);
} else {
  const e = __match_2.value;
  console.log(`Error output: ${e}`);
}
console.log("--- Ranges & Arrays ---");
for (const x of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})(1, 3)) {
  console.log(`Loop x: ${x}`);
}
const arr = ["A", "B"];
console.log(`Array: ${arr}, Fragment: ${arr[0]}`);
