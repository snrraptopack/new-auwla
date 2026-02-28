function check_status(code) {
  return (() => {
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
})();
}
function check_status_code(code) {
  return (() => {
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
})();
}
console.log(check_status_code(200));
console.log(check_status_code(404));
console.log(check_status_code(503));
console.log(check_status_code(999));
function handle_event(event) {
  return (() => {
    const __match_2 = event;
    if ((__match_2 === "click" || __match_2 === "tap" || __match_2 === "keypress")) {
      return console.log("User interaction detected!");
    }
    else if ((__match_2 === "load" || __match_2 === "unload")) {
      return console.log("Page lifecycle event!");
    }
    else if (true) {
      return console.log("Unknown event!");
    }
})();
}
handle_event("click");
handle_event("load");
handle_event("hover");
function handle_state(state) {
  return (() => {
    const __match_3 = state;
    if ((__match_3.$variant === "Loading" || __match_3.$variant === "Offline")) {
      return console.log("App is not interactive right now.");
    }
    else if (__match_3.$variant === "Ready") {
      return console.log("App is ready!");
    }
    else if (__match_3.$variant === "Error") {
      const msg = __match_3.$data[0];
      return console.log(`App encountered an error: ${msg}`);
    }
})();
}
handle_state({ $variant: "Loading", $data: [] });
handle_state({ $variant: "Ready", $data: [] });
handle_state({ $variant: "Offline", $data: [] });
handle_state({ $variant: "Error", $data: ["Network timeout"] });
function check_retries(current_retries, max_retries) {
  return (() => {
    const __match_4 = current_retries;
    if (true && (() => {
      const count = __match_4;
      return (count >= max_retries);
    })()) {
      const count = __match_4;
      return console.log("Failed: Too many retries!");
    }
    else if (true) {
      const count = __match_4;
      return console.log(`Retrying... attempt ${count}`);
    }
})();
}
check_retries(4, 3);
check_retries(1, 3);
