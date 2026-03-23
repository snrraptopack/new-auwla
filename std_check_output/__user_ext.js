import './std/string.js';
import './std/array.js';
import './std/number.js';
import './std/optional.js';
import './std/result.js';
import './std/dict.js';
export function _ext_usr_array__get(__self, index) {
  if (((index < 0) || (index >= __self.length))) {
    return ({ ok: false });
  }
  return ({ ok: true, value: __self[index] });
}

export function _ext_usr_array__set(__self, index, value) {
}

export function _ext_usr_array__low(__self) {
  return 0;
}

export function _ext_usr_array__high(__self) {
  return __self.length;
}

export function _ext_usr_array__last(__self) {
  return _ext_array__get(__self, (__self.length - 1));
}

export function _ext_usr_array__first(__self) {
  return _ext_array__get(__self, 0);
}

export function _ext_usr_array__is_empty(__self) {
  return (__self.length === 0);
}

export function _ext_usr_array__shuffle(__self) {
  for (let i = 0; i < __self.length; i += 1) {
    const random = Math.floor((Math.random() * __self.length));
    const temp = __self[i];
    __self[i] = __self[random];
    __self[random] = temp;
  }
}

export function _ext_usr_array__op_mul(__self, times) {
  let result = [];
  let i = 0;
  while ((i < times)) {
    result = result.concat(__self);
    i = (i + 1);
  }
  return result;
}

export function _ext_usr_array__op_plus(__self, other) {
  return __self.concat(other);
}

export function _ext_usr_array__sum(__self) {
  return __self.reduce((acc, val) => (acc + val), 0);
}

export function _ext_usr_array__max(__self) {
  let c_max = __self[0];
  for (let i = 1; i < __self.length; i += 1) {
    if ((__self[i] > c_max)) {
      c_max = __self[i];
    }
  }
  return c_max;
}

export function _ext_usr_array__min(__self) {
  let c_min = __self[0];
  for (let i = 1; i < __self.length; i += 1) {
    if ((__self[i] < c_min)) {
      c_min = __self[i];
    }
  }
  return c_min;
}

export function _ext_usr_dict__len(__self) {
  const keys = Object.keys(__self);
  let count = 0;
  for (const _ of keys) {
    count = (count + 1);
  }
  return count;
}

export function _ext_usr_dict__contains(__self, key) {
  return (key in __self);
}

export function _ext_usr_dict__remove(__self, key) {
  if ((key in __self)) {
    return Reflect.deleteProperty(__self, key);
  }
  return false;
}

export function _ext_usr_dict__clear(__self) {
  const keys = Object.keys(__self);
  for (const key of keys) {
    Reflect.deleteProperty(__self, key);
  }
}

export function _ext_usr_dict__is_empty(__self) {
  return (_ext_dict__len(__self) === 0);
}

export function _ext_usr_dict__get(__self, key) {
  if ((key in __self)) {
    return ({ ok: true, value: __self[key] });
  }
  return ({ ok: false });
}

export function _ext_usr_dict__set(__self, key, value) {
  __self[key] = value;
}

export function _ext_usr_dict__keys(__self) {
  let result = [];
  for (const [k, _] of Object.entries(__self)) {
    result.push(k);
  }
  return result;
}

export function _ext_usr_dict__values(__self) {
  let result = [];
  for (const [_, v] of Object.entries(__self)) {
    result.push(v);
  }
  return result;
}

export function _ext_usr_dict__map(__self, f) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    result[k] = f(v);
  }
  return result;
}

export function _ext_usr_dict__filter(__self, predicate) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    if (predicate(k, v)) {
      result[k] = v;
    }
  }
  return result;
}

export function _ext_usr_dict__for_each(__self, f) {
  for (const [k, v] of Object.entries(__self)) {
    f(k, v);
  }
}

export function _ext_usr_dict__merge(__self, other) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    result[k] = v;
  }
  for (const [k, v] of Object.entries(other)) {
    result[k] = v;
  }
  return result;
}

export function _ext_usr_dict__op_plus(__self, other) {
  return _ext_dict__merge(__self, other);
}

export function _ext_usr_dict__pick(__self, keys) {
  let result = {  };
  for (const key of keys) {
    if ((key in __self)) {
      result[key] = __self[key];
    }
  }
  return result;
}

export function _ext_usr_dict__omit(__self, keys) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    result[k] = v;
  }
  for (const key of keys) {
    result.remove(key);
  }
  return result;
}

export function _ext_usr_dict__find_key(__self, predicate) {
  for (const [k, v] of Object.entries(__self)) {
    if (predicate(v)) {
      return ({ ok: true, value: k });
    }
  }
  return ({ ok: false });
}

export function _ext_usr_dict__any(__self, predicate) {
  for (const [_, v] of Object.entries(__self)) {
    if (predicate(v)) {
      return true;
    }
  }
  return false;
}

export function _ext_usr_dict__every(__self, predicate) {
  for (const [_, v] of Object.entries(__self)) {
    if (!predicate(v)) {
      return false;
    }
  }
  return true;
}

export function _ext_usr_dict__count(__self, predicate) {
  let count = 0;
  for (const [k, v] of Object.entries(__self)) {
    if (predicate(k, v)) {
      count = (count + 1);
    }
  }
  return count;
}

export function _ext_usr_number__abs(__self) {
  if ((__self < 0)) {
    return (__self * -1);
  }
  return __self;
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

export function _ext_usr_number__minus(__self) {
  return (__self * -1);
}

export function _ext_usr_number__by(__self, value) {
  return (__self * value);
}

export function _ext_usr_number__add(__self, other) {
  return (__self + other);
}

export function _ext_usr_number__sub(__self, other) {
  return (__self - other);
}

export function _ext_usr_number__is_even(__self) {
  const r = (__self - (Math.floor((__self / 2)) * 2));
  return (r === 0);
}

export function _ext_usr_number__is_odd(__self) {
  const r = (__self - (Math.floor((__self / 2)) * 2));
  return (r !== 0);
}

export function _ext_usr_number__is_positive(__self) {
  return (__self > 0);
}

export function _ext_usr_number__is_negative(__self) {
  return (__self < 0);
}

export function _ext_usr_number__is_zero(__self) {
  return (__self === 0);
}

export function _ext_usr_number__clamp(__self, low, high) {
  if ((__self < low)) {
    return low;
  }
  if ((__self > high)) {
    return high;
  }
  return __self;
}

export function _ext_usr_optional__val_or(__self, default_v) {
  const __match_0 = __self;
  if (__match_0.ok) {
    const v = __match_0.value;
    return v;
  }
  else if (!__match_0.ok) {
    return default_v;
  }
}

export function _ext_usr_optional__is_some(__self) {
  const __match_1 = __self;
  if (__match_1.ok) {
    const _ = __match_1.value;
    return true;
  }
  else if (!__match_1.ok) {
    return false;
  }
}

export function _ext_usr_optional__is_none(__self) {
  const __match_2 = __self;
  if (__match_2.ok) {
    const _ = __match_2.value;
    return false;
  }
  else if (!__match_2.ok) {
    return true;
  }
}

export function _ext_usr_result__val_or(__self, default_v) {
  const __match_0 = __self;
  if (__match_0.ok) {
    const v = __match_0.value;
    return v;
  }
  else if (!__match_0.ok) {
    const _ = __match_0.value;
    return default_v;
  }
}

export function _ext_usr_result__is_ok(__self) {
  const __match_1 = __self;
  if (__match_1.ok) {
    const _ = __match_1.value;
    return true;
  }
  else if (!__match_1.ok) {
    const _ = __match_1.value;
    return false;
  }
}

export function _ext_usr_result__is_err(__self) {
  const __match_2 = __self;
  if (__match_2.ok) {
    const _ = __match_2.value;
    return false;
  }
  else if (!__match_2.ok) {
    const _ = __match_2.value;
    return true;
  }
}

export function _ext_usr_result__get_err(__self) {
  const __match_3 = __self;
  if (__match_3.ok) {
    const _ = __match_3.value;
    return ({ ok: false });
  }
  else if (!__match_3.ok) {
    const err = __match_3.value;
    return ({ ok: true, value: err });
  }
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

export function _ext_usr_string__is_empty(__self) {
  return (__self.length === 0);
}

export function _ext_usr_string__reverse(__self) {
  let result = "";
  for (let i = 0; i < __self.length; i += 1) {
    result = (__self.charAt(((__self.length - 1) - i)) + result);
  }
  return result;
}

export function _ext_usr_string__op_mul(__self, other) {
  return __self.repeat(other);
}

