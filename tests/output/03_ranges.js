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

for (const x of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})(1, 5)) {
  __print(`Inclusive: ${x}`);
}
for (const y of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc}, (_, i) => String.fromCharCode(i + sc)); }})(1, 5)) {
  __print(`Exclusive: ${y}`);
}
for (const c of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})("a", "d")) {
  __print(`Char: ${c}`);
}
const a = ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})("a", "z");
__print(`Array from range: ${a}`);
