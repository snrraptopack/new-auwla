import { __print } from './__util.js';
const scores = [85, 92, 78, 95, 88, 72, 96, 84, 91, 87];
const avg = (scores.reduce((num1, num2) => (num1 + num2)) / scores.length);
__print(avg);
