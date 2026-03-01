import { __print } from './__util.js';
function check_status(code) {
  const __match_0 = code;
  if ((__match_0 === 200 || __match_0 === 201)) {
    return "Success";
  }
  else if ((__match_0 === 400 || __match_0 === 401 || __match_0 === 403 || __match_0 === 404)) {
    return "Client Error";
  }
  else if ((__match_0 >= 500 && __match_0 <= 599)) {
    return "Server Error";
  }
  else if (true) {
    return "Unknown Error";
  }
}
function check_status_code(code) {
  const __match_1 = code;
  if ((__match_1 === 200 || __match_1 === 201)) {
    return "Success";
  }
  else if ((__match_1 === 400 || __match_1 === 401 || __match_1 === 403 || __match_1 === 404)) {
    return "Client Error";
  }
  else if ((__match_1 >= 500 && __match_1 <= 599)) {
    return "Server Error";
  }
  else if (true) {
    return "Unknown Error";
  }
}
__print(check_status_code(200));
__print(check_status_code(404));
__print(check_status_code(503));
__print(check_status_code(999));
function handle_event(event) {
  const __match_2 = event;
  if ((__match_2 === "click" || __match_2 === "tap" || __match_2 === "keypress")) {
    return __print("User interaction detected!");
  }
  else if ((__match_2 === "load" || __match_2 === "unload")) {
    return __print("Page lifecycle event!");
  }
  else if (true) {
    return __print("Unknown event!");
  }
}
handle_event("click");
handle_event("load");
handle_event("hover");
function handle_state(state) {
  const __match_3 = state;
  if ((__match_3.$variant === "Loading" || __match_3.$variant === "Offline")) {
    return __print("App is not interactive right now.");
  }
  else if (__match_3.$variant === "Ready") {
    return __print("App is ready!");
  }
  else if (__match_3.$variant === "Error") {
    const msg = __match_3.$data[0];
    return __print(`App encountered an error: ${msg}`);
  }
}
handle_state({ $variant: "Loading" });
handle_state({ $variant: "Ready" });
handle_state({ $variant: "Offline" });
handle_state({ $variant: "Error", $data: ["Network timeout"] });
function check_retries(current_retries, max_retries) {
  const __match_4 = current_retries;
  if (true && (() => {
    const count = __match_4;
    return (count >= max_retries);
  })()) {
    const count = __match_4;
    return __print("Failed: Too many retries!");
  }
  else if (true) {
    const count = __match_4;
    return __print(`Retrying... attempt ${count}`);
  }
}
check_retries(4, 3);
check_retries(1, 3);
