export function _ext_number__abs(__self) {
  if ((__self < 0)) {
    return (__self * -1);
  }
  return __self;
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

export function _ext_number__minus(__self) {
  return (__self * -1);
}

export function _ext_number__by(__self, value) {
  return (__self * value);
}

export function _ext_number__add(__self, other) {
  return (__self + other);
}

export function _ext_number__sub(__self, other) {
  return (__self - other);
}

export function _ext_number__is_even(__self) {
  const r = (__self - (Math.floor((__self / 2)) * 2));
  return (r === 0);
}

export function _ext_number__is_odd(__self) {
  const r = (__self - (Math.floor((__self / 2)) * 2));
  return (r !== 0);
}

export function _ext_number__is_positive(__self) {
  return (__self > 0);
}

export function _ext_number__is_negative(__self) {
  return (__self < 0);
}

export function _ext_number__is_zero(__self) {
  return (__self === 0);
}

export function _ext_number__clamp(__self, low, high) {
  if ((__self < low)) {
    return low;
  }
  if ((__self > high)) {
    return high;
  }
  return __self;
}

