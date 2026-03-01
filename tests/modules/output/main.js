import { __print } from '../../output/__util.js';
import { add, multiply, greet, another } from './math.js';
const sum = add(10, 32);
__print(sum);
const product = multiply(6, 7);
__print(product);
const msg = greet("Auwla");
__print(msg);
__print(another("Amihere"));
