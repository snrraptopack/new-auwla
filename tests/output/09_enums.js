const ok_status = { $variant: "Active", $data: [] };
const ok_inactive = { $variant: "Inactive", $data: [] };
const banned_status = { $variant: "Banned", $data: ["Violation of terms"] };
console.log("--- EXHAUSTIVE MATCH TESTING ---");
function check_status(status) {
  return (() => {
    const __match_0 = status;
    if (__match_0.$variant === "Active") {
      console.log("Status is Active");
      return undefined;
    }
    else if (__match_0.$variant === "Inactive") {
      console.log("Status is Inactive");
      return undefined;
    }
    else if (__match_0.$variant === "Banned") {
      const reason = __match_0.$data[0];
      console.log("Status is Banned: ");
      console.log(reason);
      return undefined;
    }
})();
}
check_status(ok_status);
check_status(ok_inactive);
check_status(banned_status);
console.log("--- DIRECT MATCH ASSIGNMENT ---");
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
console.log(message);
