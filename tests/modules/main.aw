// main.aw — imports from math.aw and uses the exported names

import { add, multiply, greet,another } from "./math";

let sum = add(10, 32);
print(sum);

let product = multiply(6, 7);
print(product);

let msg = greet("Auwla");
print(msg);

print(another("Amihere"));
