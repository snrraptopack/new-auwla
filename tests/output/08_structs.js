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

const alice = { name: "Alice", age: 30, is_active: true };
__print(alice.name);
__print(alice.age);
__print(alice.is_active);
let bob = { name: "Bob", age: 25, is_active: false };
bob.age = 26;
__print(bob.age);
const acc = { user: alice, balance: 1000.5 };
__print(acc.user.name);
__print(acc.balance);
