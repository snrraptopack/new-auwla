import { __print } from './__util.js';
const point = [10, 20];
const person = ["Alice", 30, true];
const [x, y] = point;
__print(`Point coordinates: x=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(x)}, y=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(y)}`);
const [name, age, active] = person;
__print(`Person: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)}, age ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(age)}, active: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(active)}`);
function get_user() {
  return ["Bob", 25];
}
const [user_name, user_age] = get_user();
__print(`User: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(user_name)}, age ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(user_age)}`);
const nested = [[1, 2], [3, 4]];
const [[a, b], [c, d]] = nested;
__print(`Nested: a=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(a)}, b=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(b)}, c=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(c)}, d=${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(d)}`);
function describe_point(p) {
  const __match_0 = p;
  if ((Array.isArray(__match_0) && __match_0.length === 2 && __match_0[0] === 0 && __match_0[1] === 0)) {
    __print("Origin");
  }
  else if ((Array.isArray(__match_0) && __match_0.length === 2 && true && __match_0[1] === 0)) {
    const x = __match_0[0];
    __print(`On X-axis at ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(x)}`);
  }
  else if ((Array.isArray(__match_0) && __match_0.length === 2 && __match_0[0] === 0 && true)) {
    const y = __match_0[1];
    __print(`On Y-axis at ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(y)}`);
  }
  else if ((Array.isArray(__match_0) && __match_0.length === 2 && true && true)) {
    const x = __match_0[0];
    const y = __match_0[1];
    __print(`Point at (${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(x)}, ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(y)})`);
  }
}
describe_point([0, 0]);
describe_point([5, 0]);
describe_point([0, 3]);
describe_point([10, 20]);
const mixed = ["test", 42, false, "end"];
const [s1, n, bs, s2] = mixed;
__print(`Mixed: ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(s1)}, ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(n)}, ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(b)}, ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(s2)}`);
