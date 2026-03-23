export function _ext_array__get(__self, index) {
  if (((index < 0) || (index >= __self.length))) {
    return ({ ok: false });
  }
  return ({ ok: true, value: __self[index] });
}

export function _ext_array__set(__self, index, value) {
}

export function _ext_array__low(__self) {
  return 0;
}

export function _ext_array__high(__self) {
  return __self.length;
}

export function _ext_array__last(__self) {
  return _ext_array__get(__self, (__self.length - 1));
}

export function _ext_array__first(__self) {
  return _ext_array__get(__self, 0);
}

export function _ext_array__is_empty(__self) {
  return (__self.length === 0);
}

export function _ext_array__shuffle(__self) {
  for (let i = 0; i < __self.length; i += 1) {
    const random = Math.floor((Math.random() * __self.length));
    const temp = __self[i];
    __self[i] = __self[random];
    __self[random] = temp;
  }
}

export function _ext_array__sum(__self) {
  return __self.reduce((acc, val) => (acc + val), 0);
}

export function _ext_array__max(__self) {
  let c_max = __self[0];
  for (let i = 1; i < __self.length; i += 1) {
    if ((__self[i] > c_max)) {
      c_max = __self[i];
    }
  }
  return c_max;
}

export function _ext_array__min(__self) {
  let c_min = __self[0];
  for (let i = 1; i < __self.length; i += 1) {
    if ((__self[i] < c_min)) {
      c_min = __self[i];
    }
  }
  return c_min;
}

