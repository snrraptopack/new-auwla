export function _ext_string__shout(__self) {
  return (__self + "!!!");
}

export function _ext_string__whisper(__self) {
  return (__self + "...");
}

export function _ext_string__first_n(__self, n) {
  let result = "";
  for (let i = 0; i < n; i += 1) {
    result = (result + __self.charAt(i));
  }
  return result;
}

export function _ext_string__is_empty(__self) {
  return (__self.length === 0);
}

export function _ext_string__reverse(__self) {
  let result = "";
  for (let i = 0; i < __self.length; i += 1) {
    result = (__self.charAt(((__self.length - 1) - i)) + result);
  }
  return result;
}

export function _ext_string__op_mul(__self, other) {
  return __self.repeat(other);
}

