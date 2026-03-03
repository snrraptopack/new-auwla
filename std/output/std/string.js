export function _ext_string_shout(__self) {
  return (__self + "!!!");
}

export function _ext_string_whisper(__self) {
  return (__self + "...");
}

export function _ext_string_first_n(__self, n) {
  let result = "";
  for (let i = 0; i < n; i++) {
    result = (result + __self.charAt(i));
  }
  return result;
}

export function _ext_string_is_empty(__self) {
  return (__self.length === 0);
}

export function _ext_string_reverse(__self) {
  let result = "";
  for (let i = 0; i < __self.length; i++) {
    result = (__self.charAt(((__self.length - 1) - i)) + result);
  }
  return result;
}

