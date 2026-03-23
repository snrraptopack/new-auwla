export function _ext_T_E__val_or(__self, default_v) {
  const __match_0 = __self;
  if (__match_0.ok) {
    const v = __match_0.value;
    return v;
  }
  else if (!__match_0.ok) {
    const _ = __match_0.value;
    return default_v;
  }
}

export function _ext_T_E__is_ok(__self) {
  const __match_1 = __self;
  if (__match_1.ok) {
    const _ = __match_1.value;
    return true;
  }
  else if (!__match_1.ok) {
    const _ = __match_1.value;
    return false;
  }
}

export function _ext_T_E__is_err(__self) {
  return !__self.is_ok();
}

export function _ext_T_E__get_err(__self) {
  const __match_2 = __self;
  if (__match_2.ok) {
    const _ = __match_2.value;
    return ({ ok: false });
  }
  else if (!__match_2.ok) {
    const e = __match_2.value;
    return ({ ok: true, value: e });
  }
}

