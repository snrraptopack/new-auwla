import { __print } from './__util.js';

import './std/string.js';
import './std/array.js';
import './std/number.js';
export function _ext_usr_User__greet(__self) {
  return ("Hello, " + __self.name);
}

export function _ext_usr_array__last(__self) {
  if ((__self.length > 0)) {
    return _ext_array__get(__self, (__self.length - 1));
  }
  return ({ ok: false });
}

export function _ext_usr_array__first(__self) {
  return _ext_array__get(__self, 0);
}

export function _ext_usr_array__low(__self) {
  return 0;
}

export function _ext_usr_array__high(__self) {
  return __self.length;
}

export function _ext_usr_array__max(__self) {
  let c_max = 0;
  for (const i of __self) {
    if ((i > c_max)) {
      c_max = i;
    }
  }
  return c_max;
}

export function _ext_usr_number__double(__self) {
  return (__self * 2);
}

export function _ext_usr_number__square(__self) {
  return (__self * __self);
}

export function _ext_usr_number__triple(__self) {
  return (__self * 3);
}

export function _ext_usr_number__by(__self, value) {
  return (__self * value);
}

export function _ext_usr_number__double_then_square(__self) {
  return _ext_number__double(__self).square();
}

export function _ext_usr_number__add(__self, other) {
  return (__self + other);
}

export function _ext_usr_string__shout(__self) {
  return (__self + "!!!");
}

export function _ext_usr_string__whisper(__self) {
  return (__self + "...");
}

export function _ext_usr_string__first_n(__self, n) {
  let result = "";
  for (let i = 0; i < n; i += 1) {
    result = (result + __self.charAt(i));
  }
  return result;
}

export function _ext_usr_number__multi_add(__self, ...others) {
  let res = __self;
  for (const o of others) {
    res += o;
  }
  return res;
}

export function _ext_usr_number__add_many(__self, ...others) {
  let res = __self;
  for (const o of others) {
    res += o;
  }
  return res;
}

export function _ext_usr_number__is_multiple_of(__self, n) {
  const div = Math.floor((__self / n));
  const rem = (__self - (div * n));
  return (rem === 0);
}

export function _ext_usr_array__sum_items(__self) {
  let total = 0;
  for (const n of __self) {
    total += n;
  }
  return total;
}

export function _ext_usr_array__filter_even_nums(__self) {
  let res = [];
  for (const n of __self) {
    const div = Math.floor((n / 2));
    const is_ev = ((n - (div * 2)) === 0);
    if (is_ev) {
      res = [...res, n];
    }
  }
  return res;
}

export function _ext_usr_Vector2__length_sq(__self) {
  return ((__self.x * __self.x) + (__self.y * __self.y));
}

export function _ext_usr_Vector2__add_vec(__self, other) {
  return { x: (__self.x + other.x), y: (__self.y + other.y) };
}

export function _ext_usr_Vector2__op_plus(__self, other) {
  return { x: (__self.x + other.x), y: (__self.y + other.y) };
}

export function _ext_usr_Vector2__op_minus(__self, other) {
  return { x: (__self.x - other.x), y: (__self.y - other.y) };
}

export function _ext_usr_Vector2__op_mul(__self, scalar) {
  return { x: (__self.x * scalar), y: (__self.y * scalar) };
}

export function _ext_usr_Vector2__op_div(__self, divisor) {
  return { x: (__self.x / divisor), y: (__self.y / divisor) };
}

export function _ext_usr_array__find_one(__self, id) {
  for (const t of __self) {
    if ((t.id === id)) {
      return ({ ok: true, value: t });
    }
  }
  return ({ ok: false, value: "id not found" });
}

export function _ext_usr_array__print_summary(__self) {
  __print(`Summary of ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(__self.length)} tasks:`);
  for (const t of __self) {
    const __match_0 = t.status;
    let status_icon;
    switch (__match_0.$variant) {
      case "Pending":
        status_icon = "⏳";
        break;
      case "Done":
        status_icon = "✅";
        break;
      case "Archived":
        status_icon = "📦";
        break;
    }
    __print(`${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(status_icon)} ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(t.title)}`);
  }
}

export function _ext_usr_array__sum(__self) {
  let total = 0;
  for (const x of __self) {
    total = (total + x);
  }
  return total;
}

