function process(n) {
  if ((n > 10)) {
    return ({ ok: true, value: (n * 2) });
  } else {
    return ({ ok: false, value: "Too small" });
  }
}
const result1 = process(20);
const __match_0 = result1;
if (__match_0.ok) {
  const val = __match_0.value;
  console.log(`Success: ${val}`);
}
else if (!__match_0.ok) {
  const err = __match_0.value;
  console.log(`Error: ${err}`);
}
const __match_1 = process(5);
let msg;
if (__match_1.ok) {
  const val = __match_1.value;
  msg = `It worked: ${val}`;
}
else if (!__match_1.ok) {
  const err = __match_1.value;
  msg = `It failed: ${err}`;
}
console.log(msg);
