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

const ok_status = { $variant: "Active", $data: [] };
const ok_inactive = { $variant: "Inactive", $data: [] };
const banned_status = { $variant: "Banned", $data: ["Violation of terms"] };
__print("--- EXHAUSTIVE MATCH TESTING ---");
function check_status(status) {
  return (() => {
    const __match_0 = status;
    if (__match_0.$variant === "Active") {
      __print("Status is Active");
      return undefined;
    }
    else if (__match_0.$variant === "Inactive") {
      __print("Status is Inactive");
      return undefined;
    }
    else if (__match_0.$variant === "Banned") {
      const reason = __match_0.$data[0];
      __print("Status is Banned: ");
      __print(reason);
      return undefined;
    }
})();
}
check_status(ok_status);
check_status(ok_inactive);
check_status(banned_status);
__print("--- DIRECT MATCH ASSIGNMENT ---");
const __match_1 = banned_status;
let message;
if (__match_1.$variant === "Active") {
  message = "All good";
}
else if (__match_1.$variant === "Inactive") {
  message = "User is inactive";
}
else if (__match_1.$variant === "Banned") {
  const reason = __match_1.$data[0];
  message = reason;
}
__print(message);
