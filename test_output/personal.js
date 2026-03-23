import { __print, __range } from './__util.js';
import * as __std_array from './std/array.js';
const a = __range(1, 100, true);
const b = __std_array._ext_array__op_mul(a, 2);
const c = __std_array._ext_array__op_plus(a, b);
__print(c);
__print(b);
