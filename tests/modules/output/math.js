export function add(a, b) {
  return (a + b);
}
export function multiply(a, b) {
  return (a * b);
}
export function greet(name) {
  return (("Hello, " + name) + "!");
}
export const another = (name) => `your name is ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)}`;
