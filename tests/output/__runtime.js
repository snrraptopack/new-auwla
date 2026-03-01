export function __ext_array_len(__self) {
  return __self.length;
}

export function __ext_array_push_val(__self, val) {
  return __self.push(val);
}

export function __ext_array_is_arr(val) {
  return Array.isArray(val);
}

export function __ext_array_last(__self) {
  if ((__ext_array_len(__self) === 0)) {
    return ({ ok: false });
  }
  return ({ ok: true, value: __self[(__ext_array_len(__self) - 1)] });
}

