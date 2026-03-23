export function _ext_dict__len(__self) {
  const keys = Object.keys(__self);
  let count = 0;
  for (const _ of keys) {
    count = (count + 1);
  }
  return count;
}

export function _ext_dict__contains(__self, key) {
  return (key in __self);
}

export function _ext_dict__remove(__self, key) {
  if ((key in __self)) {
    return Reflect.deleteProperty(__self, key);
  }
  return false;
}

export function _ext_dict__clear(__self) {
  const keys = Object.keys(__self);
  for (const key of keys) {
    Reflect.deleteProperty(__self, key);
  }
}

export function _ext_dict__is_empty(__self) {
  return (_ext_dict__len(__self) === 0);
}

export function _ext_dict__get(__self, key) {
  if ((key in __self)) {
    return ({ ok: true, value: __self[key] });
  }
  return ({ ok: false });
}

export function _ext_dict__set(__self, key, value) {
  __self[key] = value;
}

export function _ext_dict__keys(__self) {
  let result = [];
  for (const [k, _] of Object.entries(__self)) {
    result.push(k);
  }
  return result;
}

export function _ext_dict__values(__self) {
  let result = [];
  for (const [_, v] of Object.entries(__self)) {
    result.push(v);
  }
  return result;
}

export function _ext_dict__map(__self, f) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    result[k] = f(v);
  }
  return result;
}

export function _ext_dict__filter(__self, predicate) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    if (predicate(k, v)) {
      result[k] = v;
    }
  }
  return result;
}

export function _ext_dict__for_each(__self, f) {
  for (const [k, v] of Object.entries(__self)) {
    f(k, v);
  }
}

export function _ext_dict__merge(__self, other) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    result[k] = v;
  }
  for (const [k, v] of Object.entries(other)) {
    result[k] = v;
  }
  return result;
}

export function _ext_dict__op_plus(__self, other) {
  return _ext_dict__merge(__self, other);
}

export function _ext_dict__pick(__self, keys) {
  let result = {  };
  for (const key of keys) {
    if ((key in __self)) {
      result[key] = __self[key];
    }
  }
  return result;
}

export function _ext_dict__omit(__self, keys) {
  let result = {  };
  for (const [k, v] of Object.entries(__self)) {
    result[k] = v;
  }
  for (const key of keys) {
    result.remove(key);
  }
  return result;
}

export function _ext_dict__find_key(__self, predicate) {
  for (const [k, v] of Object.entries(__self)) {
    if (predicate(v)) {
      return ({ ok: true, value: k });
    }
  }
  return ({ ok: false });
}

export function _ext_dict__any(__self, predicate) {
  for (const [_, v] of Object.entries(__self)) {
    if (predicate(v)) {
      return true;
    }
  }
  return false;
}

export function _ext_dict__every(__self, predicate) {
  for (const [_, v] of Object.entries(__self)) {
    if (!predicate(v)) {
      return false;
    }
  }
  return true;
}

export function _ext_dict__count(__self, predicate) {
  let count = 0;
  for (const [k, v] of Object.entries(__self)) {
    if (predicate(k, v)) {
      count = (count + 1);
    }
  }
  return count;
}

