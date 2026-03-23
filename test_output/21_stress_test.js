import { __print } from './__util.js';
import * as __user from './__user_ext.js';
function test_collections() {
  const base = [1, 2, 3];
  const expanded = [0, ...base, 4, 5];
  __print(`Expanded Array length: ${expanded.length}`);
  const scores = { "alice": 100, "bob": 80 };
  const more = { ...scores, "charlie": 90 };
  __print("Scores:");
  for (const [k, v] of Object.entries(more)) {
    __print(`${k}: ${v}`);
  }
}
function safe_divide(a, b) {
  if ((b === 0)) {
    return ({ ok: false, value: "Division by zero" });
  }
  return ({ ok: true, value: (a / b) });
}
function stress_main() {
  __print("--- STARTING AUWLA STRESS TEST ---");
  const start_val = 10;
  const final_val = __user._ext_usr_number__add_many(start_val, 1, 2, 3, 4);
  __print(`Extended Add (10 + 1,2,3,4) = ${final_val}`);
  __print(`Is ${final_val} multiple of 5? ${__user._ext_usr_number__is_multiple_of(final_val, 5)}`);
  const numbers = [1, 2, 3, 4, 5, 6, 7, 8];
  const evens = __user._ext_usr_array_number__filter_even_nums(numbers);
  __print(`Evens count: ${evens.length}`);
  __print(`Sum of evens: ${__user._ext_usr_array_number__sum_items(evens)}`);
  const v1 = { x: 3, y: 4 };
  const v2 = { x: 7, y: 6 };
  const v3 = __user._ext_usr_Vector2__add_vec(v1, v2);
  __print(`Vector Sum: (${v3.x}, ${v3.y}), LengthSq: ${__user._ext_usr_Vector2__length_sq(v3)}`);
  const __match_0 = safe_divide(100, 5);
  if (__match_0.ok) {
    const val = __match_0.value;
    __print(`100 / 5 = ${val}`);
  }
  else if (!__match_0.ok) {
    const err = __match_0.value;
    __print(`Error: ${err}`);
  }
  const __match_1 = safe_divide(99, 0);
  if (__match_1.ok) {
    const val = __match_1.value;
    __print(`99 / 0 = ${val}`);
  }
  else if (!__match_1.ok) {
    const err = __match_1.value;
    __print(`Error: ${err}`);
  }
  __print("Step loop (0 to 12 step 3):");
  for (let i = 0; i <= 13; i += 3) {
    __print(i);
  }
  test_collections();
  __print("--- STRESS TEST COMPLETE ---");
}
stress_main();
