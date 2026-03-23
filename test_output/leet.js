import { __print } from './__util.js';
function two_sum(nums, target) {
  const num_map = {  };
  for (let i = 0; i <= nums.length; i += 1) {
    const complement = (target - nums[i]);
    if ((complement in num_map)) {
      return [num_map[complement], i];
    }
    num_map[nums[i]] = i;
  }
  return [];
}
const nums1 = [2, 7, 11, 15];
const target1 = 9;
const result1 = two_sum(nums1, target1);
__print(`Two Sum Test 1: ${result1}`);
const nums2 = [3, 2, 4];
const target2 = 6;
const result2 = two_sum(nums2, target2);
__print(`Two Sum Test 2: ${result2}`);
const nums3 = [3, 3];
const target3 = 6;
const result3 = two_sum(nums3, target3);
__print(`Two Sum Test 3: ${result3}`);
function is_valid_parentheses(s) {
  const stack = [];
  const pairs = { ")": "(", "]": "[", "}": "{" };
  for (const ch of s) {
    if ((ch in pairs)) {
      if (((stack.length === 0) || (stack[(stack.length - 1)] !== pairs[ch]))) {
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
__print(`() : ${is_valid_parentheses("()")}`);
__print(`()[] : ${is_valid_parentheses("()[]{}")}`);
__print(`(] : ${is_valid_parentheses("(]")}`);
__print(`([)] : ${is_valid_parentheses("([)]")}`);
__print(`${[]} : ${is_valid_parentheses("{[]}")}`);
function max_subarray_sum(arr, k) {
  if ((arr.length < k)) {
    return 0;
  }
  let max_sum = 0;
  let window_sum = 0;
  for (let i = 0; i <= k; i += 1) {
    window_sum = (window_sum + arr[i]);
  }
  max_sum = window_sum;
  for (let i = k; i <= arr.length; i += 1) {
    window_sum = ((window_sum - arr[(i - k)]) + arr[i]);
    if ((window_sum > max_sum)) {
      max_sum = window_sum;
    }
  }
  return max_sum;
}
__print("\nMax Subarray Sum Tests:");
const arr1 = [2, 1, 5, 1, 3, 2];
const k1 = 3;
__print(`Array: ${arr1}, k=${k1}, Max Sum: ${max_subarray_sum(arr1, k1)}`);
const arr2 = [2, 3, 4, 1, 5];
const k2 = 2;
__print(`Array: ${arr2}, k=${k2}, Max Sum: ${max_subarray_sum(arr2, k2)}`);
function reverse_string(s) {
  let chars = s.split("");
  let left = 0;
  let right = (chars.length - 1);
  while ((left < right)) {
    const temp = chars[left];
    chars[left] = chars[right];
    chars[right] = temp;
    left = (left + 1);
    right = (right - 1);
  }
  return chars.join("");
}
__print("\nReverse String Tests:");
__print(`hello -> ${reverse_string("hello")}`);
__print(`Auwla -> ${reverse_string("Auwla")}`);
__print(`racecar -> ${reverse_string("racecar")}`);
function factorial(n) {
  if ((n <= 1)) {
    return 1;
  }
  return (n * factorial((n - 1)));
}
__print("\nFactorial Tests:");
__print(`factorial(5) = ${factorial(5)}`);
__print(`factorial(0) = ${factorial(0)}`);
__print(`factorial(7) = ${factorial(7)}`);
