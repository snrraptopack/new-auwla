let i = 0;
while ((i < 3)) {
  console.log(`While Loop i: ${i}`);
  i = (i + 1);
}
const items = ["apple", "banana", "cherry"];
for (const item of items) {
  console.log(`For Loop item: ${item}`);
}
for (const num of ((__s, __e) => {if (typeof __s === 'number') return Array.from({length: __e - __s + 1}, (_, i) => i + __s); else { const sc = __s.charCodeAt(0), ec = __e.charCodeAt(0); return Array.from({length: ec - sc + 1}, (_, i) => String.fromCharCode(i + sc)); }})(5, 7)) {
  console.log(`For Loop range: ${num}`);
}
