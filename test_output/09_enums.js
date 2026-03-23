import { __print } from './__util.js';
const ok_status = { $variant: "Active" };
const ok_inactive = { $variant: "Inactive" };
const banned_status = { $variant: "Banned", $data: ["Violation of terms"] };
__print("--- EXHAUSTIVE MATCH TESTING ---");
function check_status(status) {
  const __match_0 = status;
  switch (__match_0.$variant) {
    case "Active":
      __print("Status is Active");
      break;
    case "Inactive":
      __print("Status is Inactive");
      break;
    case "Banned":
      const reason = __match_0.$data[0];
      __print("Status is Banned: ");
      __print(reason);
      break;
  }
}
check_status(ok_status);
check_status(ok_inactive);
check_status(banned_status);
__print("--- DIRECT MATCH ASSIGNMENT ---");
const __match_1 = banned_status;
let message;
switch (__match_1.$variant) {
  case "Active":
    message = "All good";
    break;
  case "Inactive":
    message = "User is inactive";
    break;
  case "Banned":
    const reason = __match_1.$data[0];
    message = reason;
    break;
}
__print(message);
