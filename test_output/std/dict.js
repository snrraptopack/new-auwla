export function _ext_dict__get(__self, key) {
  if (__self.has(key)) {
    return ({ ok: true, value: __self[key] });
  }
  return ({ ok: false });
}

export function _ext_dict__set(__self, key, value) {
  __self[key] = value;
}

export function _ext_dict__is_empty(__self) {
  return (__self.size === 0);
}

