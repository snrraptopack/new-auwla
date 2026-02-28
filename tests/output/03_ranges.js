for (const x of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})(1, 5)) {
  console.log(`Inclusive: ${x}`);
}
for (const y of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc}, (_, i) => String.fromCharCode(i + sc)); }})(1, 5)) {
  console.log(`Exclusive: ${y}`);
}
for (const c of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})("a", "d")) {
  console.log(`Char: ${c}`);
}
const a = ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})("a", "z");
console.log(`Array from range: ${a}`);
