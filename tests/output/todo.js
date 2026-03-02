import { __print } from './__util.js';
import * as __auwla from './__runtime.js';
function main() {
  const my_tasks = [{ id: 101, title: "Refactor Codegen", status: { $variant: "Done" } }, { id: 102, title: "Secure JS Interop", status: { $variant: "Pending" } }, { id: 103, title: "Ship Auwla", status: { $variant: "Pending" } }];
  __print("--- AUWLA TASK MANAGER ---");
  __auwla._ext_array_Task_print_summary(my_tasks);
  __print("Searching for Task 102...");
  const __match_1 = __auwla._ext_array_Task_find_one(my_tasks, 1022);
  if (__match_1.ok) {
    const t = __match_1.value;
    __print("Target Found!");
    __print(`ID: ${t.id}`);
    __print(`Label: ${t.title}`);
    if ((t.id > 100)) {
      __print("Priority: High (Legacy System)");
    }
  }
  else if (!__match_1.ok) {
    const msg = __match_1.value;
    __print(`Error: ${msg}`);
  }
  const version = "v1.0.0";
  __print("Checking version prefix...");
  const __match_2 = __auwla._ext_string_get(version, 0);
  if (__match_2.ok) {
    const c = __match_2.value;
    if ((c === "v")) {
      __print("Version starts with 'v' - Valid.");
    } else {
      __print(`Unknown version format: ${c}`);
    }
  }
  else if (!__match_2.ok) {
    __print("Empty version string detected.");
  }
}
main();
