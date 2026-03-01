import { __print } from './__util.js';
let i = 0;
while ((i < 3)) {
  __print(`While Loop i: ${i}`);
  i = (i + 1);
}
const items = ["apple", "banana", "cherry"];
for (const item of items) {
  __print(`For Loop item: ${item}`);
}
for (let num = 5; num <= 7; num++) {
  __print(`For Loop range: ${num}`);
}
