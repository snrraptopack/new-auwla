import { __print } from './__util.js';
import * as __std_optional from './std/optional.js';
import * as __std_array from './std/array.js';
import * as __std_dict from './std/dict.js';
function two_sum(nums, target) {
  const num_map = {  };
  for (let i = 0; i <= nums.length; i += 1) {
    const complement = (target - ((_o) => _o.ok ? _o.value : (0))(__std_array._ext_array__get(nums, i)));
    if ((complement in num_map)) {
      return [((_o) => _o.ok ? _o.value : (0))(__std_dict._ext_dict__get(num_map, complement)), i];
    }
    __std_dict._ext_dict__set(num_map, ((_o) => _o.ok ? _o.value : (0))(__std_array._ext_array__get(nums, i)), i);
  }
  return [];
}
const nums1 = [2, 7, 11, 15];
const target1 = 9;
const result1 = two_sum(nums1, target1);
__print(`Two Sum Test 1: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(result1)}`);
const nums2 = [3, 2, 4];
const target2 = 6;
const result2 = two_sum(nums2, target2);
__print(`Two Sum Test 2: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(result2)}`);
const nums3 = [3, 3];
const target3 = 6;
const result3 = two_sum(nums3, target3);
__print(`Two Sum Test 3: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(result3)}`);
function is_valid_parentheses(s) {
  const stack = [];
  const pairs = { ")": "(", "]": "[", "}": "{" };
  for (const ch of s) {
    if ((ch in pairs)) {
      if (((stack.length === 0) || (__std_optional._ext_optional__val_or(__std_array._ext_array__get(stack, (stack.length - 1)), " ") !== __std_optional._ext_optional__val_or(__std_dict._ext_dict__get(pairs, ch), " ")))) {
        return false;
      }
      ((_r) => _r != null ? { ok: true, value: _r } : { ok: false })(stack.pop());
    } else {
      stack.push(ch);
    }
  }
  return (stack.length === 0);
}
__print("Valid Parentheses Tests:");
__print(`() : ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(is_valid_parentheses("()"))}`);
__print(`()[] : ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(is_valid_parentheses("()[]{}"))}`);
__print(`(] : ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(is_valid_parentheses("(]"))}`);
__print(`([)] : ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(is_valid_parentheses("([)]"))}`);
__print(`${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)([])} : ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(is_valid_parentheses("{[]}"))}`);
function max_subarray_sum(arr, k) {
  if ((arr.length < k)) {
    return 0;
  }
  let max_sum = 0;
  let window_sum = 0;
  for (let i = 0; i <= k; i += 1) {
    window_sum = (window_sum + __std_optional._ext_optional__val_or(__std_array._ext_array__get(arr, i), 0));
  }
  max_sum = window_sum;
  for (let i = k; i <= arr.length; i += 1) {
    window_sum = ((window_sum - __std_optional._ext_optional__val_or(__std_array._ext_array__get(arr, (i - k)), 0)) + __std_optional._ext_optional__val_or(__std_array._ext_array__get(arr, i), 0));
    if ((window_sum > max_sum)) {
      max_sum = window_sum;
    }
  }
  return max_sum;
}
__print("\nMax Subarray Sum Tests:");
const arr1 = [2, 1, 5, 1, 3, 2];
const k1 = 3;
__print(`Array: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(arr1)}, k=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(k1)}, Max Sum: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(max_subarray_sum(arr1, k1))}`);
const arr2 = [2, 3, 4, 1, 5];
const k2 = 2;
__print(`Array: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(arr2)}, k=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(k2)}, Max Sum: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(max_subarray_sum(arr2, k2))}`);
function reverse_string(s) {
  let chars = s.split("");
  let left = 0;
  let right = (chars.length - 1);
  while ((left < right)) {
    const temp = __std_optional._ext_optional__val_or(__std_array._ext_array__get(chars, left), "");
    __std_array._ext_array__set(chars, left, __std_optional._ext_optional__val_or(__std_array._ext_array__get(chars, right), ""));
    __std_array._ext_array__set(chars, right, temp);
    left = (left + 1);
    right = (right - 1);
  }
  return chars.join("");
}
__print("\nReverse String Tests:");
__print(`hello -> ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(reverse_string("hello"))}`);
__print(`Auwla -> ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(reverse_string("Auwla"))}`);
__print(`racecar -> ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(reverse_string("racecar"))}`);
function factorial(n) {
  if ((n <= 1)) {
    return 1;
  }
  return (n * factorial((n - 1)));
}
__print("\nFactorial Tests:");
__print(`factorial(5) = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(factorial(5))}`);
__print(`factorial(0) = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(factorial(0))}`);
__print(`factorial(7) = ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(factorial(7))}`);
const one = [1];
const two = [1];
