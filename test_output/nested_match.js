import { __print } from './__util.js';
const user = { name: "Ama", address: { city: "Tarkwa", country: "Ghana" } };
const __match_0 = user;
if ((__match_0.name !== undefined && (__match_0.address.city === "Accra"))) {
  const name = __match_0.name;
  __print(`${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)} is in Accra`);
}
else if ((__match_0.name !== undefined && (__match_0.address.city !== undefined))) {
  const name = __match_0.name;
  const city = __match_0.address.city;
  __print(`${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(name)} is in ${((_v) => typeof _v === 'object' && _v !== null ? JSON.stringify(_v) : _v)(city)}`);
}
