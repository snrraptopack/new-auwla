import { __print } from './__util.js';
function print(message) {
  __print(message);
  return;
}
function max2(a, b) {
  return Math.max(a, b);
}
function to_upper(s) {
  return s.toUpperCase();
}
function strlen(s) {
  return s.length;
}
function tag(msg) {
  return ("[auwla] " + msg);
}
function main() {
  print(tag("global fn test"));
  const n = max2(3, 11);
  print(`max2 = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(n)}`);
  const loud = to_upper("auwla");
  print(`loud = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(loud)}`);
  const l = strlen("hello");
  print(`strlen = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(l)}`);
}
main();
