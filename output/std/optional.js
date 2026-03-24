export function _ext_optional__val_or(__self, default_v) {
  const __match_0 = __self;
  if (__match_0.ok) {
    const v = __match_0.value;
    return v;
  }
  else if (!__match_0.ok) {
    return default_v;
  }
}

export function _ext_optional__is_some(__self) {
  const __match_1 = __self;
  if (__match_1.ok) {
    const _ = __match_1.value;
    return true;
  }
  else if (!__match_1.ok) {
    return false;
  }
}

export function _ext_optional__is_none(__self) {
  const __match_2 = __self;
  if (__match_2.ok) {
    const _ = __match_2.value;
    return false;
  }
  else if (!__match_2.ok) {
    return true;
  }
}

