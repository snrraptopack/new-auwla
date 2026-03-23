import { __print } from './__util.js';
const name = "Auwla";
const age = 42;
const sentence = `${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)} is ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(age)} years old.`;
__print(sentence);
__print(`Math inside string: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)((10 + 20))}`);
