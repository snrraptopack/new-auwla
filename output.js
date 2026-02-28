const num = 42;
let a = "hello";
a = "world";
function checked(n) {
  if ((n > 10)) {
    return ({ ok: true, value: "Too big!" });
  } else {
    return ({ ok: false, value: "Okay!" });
  }
}
console.log("Number:", num);
console.log("String:", a);
console.log("Bool:", true);
console.log("Expr:", (10 + (20 * 2)));
const result = checked(num);
const __match_0 = result;
let msg;
if (__match_0.ok) {
  const val = __match_0.value;
  msg = val;
} else {
  const err = __match_0.value;
  msg = err;
}
console.log("Match result:", msg);
let result2 = checked(5);
const __match_1 = result2;
let msg2;
if (__match_1.ok) {
  const val = __match_1.value;
  const upper = val;
  msg2 = upper;
} else {
  const err = __match_1.value;
  msg2 = err;
}
console.log("Match block:", msg2);
let result3 = checked(3);
const __match_2 = result3;
if (__match_2.ok) {
  const val = __match_2.value;
  console.log("Got value:", val);
} else {
  const err = __match_2.value;
  console.log("Got error:", err);
}
