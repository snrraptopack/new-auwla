export function _ext_array_low(__self) {
  return 0;
}

export function _ext_array_high(__self) {
  return __self.length;
}

export function _ext_array_last(__self) {
  if ((__self.length > 0)) {
    return ({ ok: true, value: __self[(__self.length - 1)] });
  }
  return ({ ok: false });
}

export function _ext_array_first(__self) {
  if ((__self.length > 0)) {
    return ({ ok: true, value: __self[0] });
  }
  return ({ ok: false });
}

export function _ext_array_is_empty(__self) {
  return (__self.length === 0);
}

export function _ext_array_shuffle(__self) {
  for (let i = 0; i < __self.length; i++) {
    const random = Math.floor((Math.random() * __self.length));
    const temp = __self[i];
    __self[i] = __self[random];
    __self[random] = temp;
  }
}

export function _ext_array_number_sum(__self) {
  return __self.reduce((acc, val) => (acc + val), 0);
}

export function _ext_array_number_max(__self) {
  let c_max = __self[0];
  for (let i = 1; i < __self.length; i++) {
    if ((__self[i] > c_max)) {
      c_max = __self[i];
    }
  }
  return c_max;
}

export function _ext_array_number_min(__self) {
  let c_min = __self[0];
  for (let i = 1; i < __self.length; i++) {
    if ((__self[i] < c_min)) {
      c_min = __self[i];
    }
  }
  return c_min;
}

