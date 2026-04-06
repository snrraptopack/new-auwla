import { __print } from './__util.js';
const valid_json = "[1, 2, 3, 4, 5]";
const valid = JSON.parse(valid_json);
const __match_0 = valid;
if (__match_0.ok) {
  const obj = __match_0.value;
  __print(`Success parsing array: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(obj)}`);
}
else if (!__match_0.ok) {
  const err = __match_0.value;
  __print(`Error: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(err)}`);
}
const a = Math.random();
__print(`Random number: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(a)}`);
const invalid = JSON.parse("not valid json at all");
const __match_1 = invalid;
if (__match_1.ok) {
  const obj = __match_1.value;
  __print(`Success: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(obj)}`);
}
else if (!__match_1.ok) {
  const err = __match_1.value;
  __print(`Error caught: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(err)}`);
}
const null_test = JSON.parse("null");
const __match_2 = null_test;
if (__match_2.ok) {
  const result = __match_2.value;
  __print("Parsed null successfully");
}
else if (!__match_2.ok) {
  const err = __match_2.value;
  __print(`Error parsing null: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(err)}`);
}
const number_test = JSON.parse("42");
const __match_3 = number_test;
if (__match_3.ok) {
  const num = __match_3.value;
  __print(`Parsed number: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(num)}`);
}
else if (!__match_3.ok) {
  const err = __match_3.value;
  __print(`Error: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(err)}`);
}
