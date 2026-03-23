export function _ext_dict_K_V__get(__self, key) {
  if (__self.contains(key)) {
    return ({ ok: true, value: __self[key] });
  }
  return ({ ok: false });
}

export function _ext_dict_K_V__set(__self, key, value) {
  __self[key] = value;
}

export function _ext_dict_K_V__is_empty(__self) {
  return (__self.len() === 0);
}

