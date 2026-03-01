export function __ext_array_length(__self) {
  return __self.length;
}

export function __ext_array_last(__self) {
  if ((__self.length > 0)) {
    return ({ ok: true, value: __self[(__self.length - 1)] });
  }
  return ({ ok: false });
}

export function __ext_array_first(__self) {
  if ((__self.length > 0)) {
    return ({ ok: true, value: __self[0] });
  }
  return ({ ok: false });
}

