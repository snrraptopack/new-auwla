function __print(...args) {
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

function process(n) {
  if ((n > 10)) {
    return ({ ok: true, value: (n * 2) });
  } else {
    return ({ ok: false, value: "Too small" });
  }
}
const result1 = process(20);
const __match_0 = result1;
if (__match_0.ok) {
  const val = __match_0.value;
  __print(`Success: ${val}`);
}
else if (!__match_0.ok) {
  const err = __match_0.value;
  __print(`Error: ${err}`);
}
const __match_1 = process(5);
let msg;
if (__match_1.ok) {
  const val = __match_1.value;
  msg = `It worked: ${val}`;
}
else if (!__match_1.ok) {
  const err = __match_1.value;
  msg = `It failed: ${err}`;
}
__print(msg);
