export function __print(...args) {
  const format = (val, top = false) => {
    if (val && typeof val === 'object' && 'ok' in val) {
      if (val.ok) return `some(${format(val.value)})`;
      if ('value' in val) return `none(${format(val.value)})`;
      return 'none';
    }
    if (Array.isArray(val)) return `[${val.map(v => format(v)).join(', ')}]`;
    if (typeof val === 'string' && !top) return `"${val}"`;
    if (typeof val === 'object' && val !== null) {
      const props = Object.entries(val).map(([k, v]) => `${k}: ${format(v)}`).join(', ');
      return `{ ${props} }`;
    }
    return val;
  };
  console.log(...args.map(a => format(a, true)));
}

export function __range(s, e, inclusive) {
  if (typeof s === 'number') {
    return Array.from({length: e - s + (inclusive ? 1 : 0)}, (_, i) => i + s);
  } else {
    const sc = s.charCodeAt(0), ec = e.charCodeAt(0);
    return Array.from({length: ec - sc + (inclusive ? 1 : 0)}, (_, i) => String.fromCharCode(i + sc));
  }
}
