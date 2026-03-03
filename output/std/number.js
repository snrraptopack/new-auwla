export function _ext_number_abs(__self) {
  if ((__self < 0)) {
    return (__self * -1);
  }
  return __self;
}

export function _ext_number_double(__self) {
  return (__self * 2);
}

export function _ext_number_square(__self) {
  return (__self * __self);
}

export function _ext_number_triple(__self) {
  return (__self * 3);
}

export function _ext_number_by(__self, value) {
  return (__self * value);
}

export function _ext_number_add(__self, other) {
  return (__self + other);
}

export function _ext_number_sub(__self, other) {
  return (__self - other);
}

export function _ext_number_is_even(__self) {
  const r = (__self - (Math.floor((__self / 2)) * 2));
  return (r === 0);
}

export function _ext_number_is_odd(__self) {
  const r = (__self - (Math.floor((__self / 2)) * 2));
  return (r !== 0);
}

export function _ext_number_is_positive(__self) {
  return (__self > 0);
}

export function _ext_number_is_negative(__self) {
  return (__self < 0);
}

export function _ext_number_is_zero(__self) {
  return (__self === 0);
}

export function _ext_number_clamp(__self, low, high) {
  if ((__self < low)) {
    return low;
  }
  if ((__self > high)) {
    return high;
  }
  return __self;
}

