import { __print } from './__util.js';

import './std/string.js';
import './std/array.js';
import './std/number.js';
export function _ext_User__greet(__self) {
  return ("Hello, " + __self.name);
}

export function _ext_array__last(__self) {
  if ((__self.length > 0)) {
    return ({ ok: true, value: __self[(__self.length - 1)] });
  }
  return ({ ok: false });
}

export function _ext_array__first(__self) {
  if ((__self.length > 0)) {
    return ({ ok: true, value: __self[0] });
  }
  return ({ ok: false });
}

export function _ext_array__low(__self) {
  return 0;
}

export function _ext_array__high(__self) {
  return __self.length;
}

export function _ext_array_number__max(__self) {
  let c_max = 0;
  for (let i = _ext_array__low(__self); i < _ext_array__high(__self); i++) {
    if ((__self[i] > c_max)) {
      c_max = __self[i];
    }
  }
  return c_max;
}

export function _ext_number__double(__self) {
  return (__self * 2);
}

export function _ext_number__square(__self) {
  return (__self * __self);
}

export function _ext_number__triple(__self) {
  return (__self * 3);
}

export function _ext_number__by(__self, value) {
  return (__self * value);
}

export function _ext_number__double_then_square(__self) {
  return _ext_number__square(_ext_number__double(__self));
}

export function _ext_number__add(__self, other) {
  return (__self + other);
}

export function _ext_string__shout(__self) {
  return (__self + "!!!");
}

export function _ext_string__whisper(__self) {
  return (__self + "...");
}

export function _ext_string__first_n(__self, n) {
  let result = "";
  for (let i = 0; i < n; i++) {
    result = (result + __self.charAt(i));
  }
  return result;
}

export function _ext_array__shuffle(__self) {
  for (let i = 0; i < __self.length; i++) {
    const random = Math.floor((Math.random() * __self.length));
    const temp = __self[i];
    __self[i] = __self[random];
    __self[random] = temp;
  }
}

export function _ext_string__ones(__self) {
  return __self;
}

export function _ext_array_Task__find_one(__self, id) {
  for (const t of __self) {
    if ((t.id === id)) {
      return ({ ok: true, value: t });
    }
  }
  return ({ ok: false, value: "id not found" });
}

export function _ext_array_Task__print_summary(__self) {
  __print(`Summary of ${__self.length} tasks:`);
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
    __print(`${status_icon} ${t.title}`);
  }
}

export function _ext_array_number__sum(__self) {
  let total = 0;
  for (const x of __self) {
    total = (total + x);
  }
  return total;
}

