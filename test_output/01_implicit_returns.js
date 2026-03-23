import { __print } from './__util.js';
function add(a, b) {
  return (a + b);
}
function get_name() {
  return "Auwla";
}
const result = add(10, 20);
__print(`Result: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(result)}`);
