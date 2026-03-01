export function _ext_array_len(__self) {
  return __self.length;
}

export function _ext_array_push_val(__self, val) {
  return __self.push(val);
}

export function _ext_array_is_arr(val) {
  return Array.isArray(val);
}

export function _ext_array_last(__self) {
  if ((_ext_array_len(__self) === 0)) {
    return ({ ok: false });
  }
  return ({ ok: true, value: __self[(_ext_array_len(__self) - 1)] });
}

export function _ext_array_low(__self) {
  return 0;
}

export function _ext_array_high(__self) {
  return _ext_array_len(__self);
}

export function _ext_array_number_max(__self) {
  let c_max = 0;
  for (let i = _ext_array_low(__self); i < _ext_array_high(__self); i++) {
    if ((__self[i] > c_max)) {
      c_max = __self[i];
    }
  }
  return c_max;
}

export function _ext_string_repeat(__self, times) {
  return __self.repeat(times);
}

