import { __print } from './__util.js';
let i = 0;
while ((i < 3)) {
  __print(`While Loop i: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(i)}`);
  i = (i + 1);
}
const items = ["apple", "banana", "cherry"];
for (const item of items) {
  __print(`For Loop item: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(item)}`);
}
for (let num = 5; num <= 7; num += 1) {
  __print(`For Loop range: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(num)}`);
}
