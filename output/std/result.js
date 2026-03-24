export function _ext_result__val_or(__self, default_v) {
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

export function _ext_result__is_ok(__self) {
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

export function _ext_result__is_err(__self) {
  const __match_2 = __self;
  if (__match_2.ok) {
    const _ = __match_2.value;
    return false;
  }
  else if (!__match_2.ok) {
    const _ = __match_2.value;
    return true;
  }
}

export function _ext_result__get_err(__self) {
  const __match_3 = __self;
  if (__match_3.ok) {
    const _ = __match_3.value;
    return ({ ok: false });
  }
  else if (!__match_3.ok) {
    const err = __match_3.value;
    return ({ ok: true, value: err });
  }
}

