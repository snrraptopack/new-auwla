import './std/string.js';
import './std/number.js';
export function _ext_usr_array_T__get(__self, index) {
  if (((index < 0) || (index >= __self.len()))) {
    return ({ ok: false });
  }
  return ({ ok: true, value: __self[index] });
}

export function _ext_usr_array_T__low(__self) {
  return 0;
}

export function _ext_usr_array_T__high(__self) {
  return __self.len();
}

export function _ext_usr_array_T__last(__self) {
  return __self.get((__self.len() - 1));
}

export function _ext_usr_array_T__first(__self) {
  return __self.get(0);
}

export function _ext_usr_array_T__is_empty(__self) {
  return (__self.len() === 0);
}

export function _ext_usr_array_T__shuffle(__self) {
  for (let i = 0; i < __self.len(); i += 1) {
    const random = Math.floor((Math.random() * __self.len()));
    const temp = __self[i];
    __self[i] = __self[random];
    __self[random] = temp;
  }
}

export function _ext_usr_array_number__sum(__self) {
  return __self.reduce((acc, val) => (acc + val), 0);
}

export function _ext_usr_array_number__max(__self) {
  let c_max = __self[0];
  for (let i = 1; i < __self.len(); i += 1) {
    if ((__self[i] > c_max)) {
      c_max = __self[i];
    }
  }
  return c_max;
}

export function _ext_usr_array_number__min(__self) {
  let c_min = __self[0];
  for (let i = 1; i < __self.len(); i += 1) {
    if ((__self[i] < c_min)) {
      c_min = __self[i];
    }
  }
  return c_min;
}

export function _ext_usr_dict_K_V__get(__self, key) {
  if (__self.contains(key)) {
    return ({ ok: true, value: __self[key] });
  }
  return ({ ok: false });
}

export function _ext_usr_dict_K_V__set(__self, key, value) {
  __self[key] = value;
}

export function _ext_usr_dict_K_V__is_empty(__self) {
  return (__self.len() === 0);
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

export function _ext_usr_T__val_or(__self, default_v) {
  const __match_0 = __self;
  if (__match_0.ok) {
    const v = __match_0.value;
    return v;
  }
  else if (!__match_0.ok) {
    return default_v;
  }
}

export function _ext_usr_T__is_some(__self) {
  const __match_1 = __self;
  if (__match_1.ok) {
    const _ = __match_1.value;
    return true;
  }
  else if (!__match_1.ok) {
    return false;
  }
}

export function _ext_usr_T__is_none(__self) {
  return !__self.is_some();
}

export function _ext_usr_T_E__val_or(__self, default_v) {
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

export function _ext_usr_T_E__is_ok(__self) {
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

export function _ext_usr_T_E__is_err(__self) {
  return !__self.is_ok();
}

export function _ext_usr_T_E__get_err(__self) {
  const __match_2 = __self;
  if (__match_2.ok) {
    const _ = __match_2.value;
    return ({ ok: false });
  }
  else if (!__match_2.ok) {
    const e = __match_2.value;
    return ({ ok: true, value: e });
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

