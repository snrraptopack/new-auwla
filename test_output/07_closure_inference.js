import { __print } from './__util.js';
const nums = [1, 2, 3, 4, 5];
const doubled = nums.map((x) => (x * 2));
const greater_than_two = nums.filter((x) => (x > 2));
const sum = nums.reduce((acc, val) => (acc + val), 0);
__print("Doubled:", doubled);
__print("GreaterThanTwo:", greater_than_two);
__print("Sum:", sum);
