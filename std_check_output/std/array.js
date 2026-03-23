export function _ext_array_T__get(__self, index) {
  if (((index < 0) || (index >= __self.len()))) {
    return ({ ok: false });
  }
  return ({ ok: true, value: __self[index] });
}

export function _ext_array_T__low(__self) {
  return 0;
}

export function _ext_array_T__high(__self) {
  return __self.len();
}

export function _ext_array_T__last(__self) {
  return __self.get((__self.len() - 1));
}

export function _ext_array_T__first(__self) {
  return __self.get(0);
}

export function _ext_array_T__is_empty(__self) {
  return (__self.len() === 0);
}

export function _ext_array_T__shuffle(__self) {
  for (let i = 0; i < __self.len(); i += 1) {
    const random = Math.floor((Math.random() * __self.len()));
    const temp = __self[i];
    __self[i] = __self[random];
    __self[random] = temp;
  }
}

export function _ext_array_number__sum(__self) {
  return __self.reduce((acc, val) => (acc + val), 0);
}

export function _ext_array_number__max(__self) {
  let c_max = __self[0];
  for (let i = 1; i < __self.len(); i += 1) {
    if ((__self[i] > c_max)) {
      c_max = __self[i];
    }
  }
  return c_max;
}

export function _ext_array_number__min(__self) {
  let c_min = __self[0];
  for (let i = 1; i < __self.len(); i += 1) {
    if ((__self[i] < c_min)) {
      c_min = __self[i];
    }
  }
  return c_min;
}

