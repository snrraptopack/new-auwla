import { __print } from './__util.js';
import * as __std_dict from './std/dict.js';
import * as __std_optional from './std/optional.js';
function test_nested_collections() {
  __print("--- Test 1: Nested Collections ---");
  let registry = {  };
  __std_dict._ext_dict__set(registry, "primes", [2, 3, 5]);
  __std_dict._ext_dict__set(registry, "fib", [1, 1, 2, 3]);
  __print("registry-primes", __std_dict._ext_dict__get(registry, "primes"));
  if (("primes" in registry)) {
    const p = __std_dict._ext_dict__get(registry, "primes");
    __print(`Primes: ${p}`);
  }
  let total_elements = 0;
  const keys = ["primes", "fib"];
  for (const key of keys) {
    if ((key in registry)) {
      total_elements = (total_elements + __std_optional._ext_optional__val_or(__std_dict._ext_dict__get(registry, key), []).length);
    }
  }
  __print(`Total elements: ${total_elements}`);
}
function test_mutability_in_loops() {
  __print("\n--- Test 2: Mutability in Loops ---");
  let sum = 0;
  for (let i = 1; i <= 5; i += 1) {
    const square = (i * i);
    let running_avg = (square + sum);
    if (((i % 2) === 0)) {
      running_avg = (running_avg / 2);
    }
    sum = (sum + running_avg);
    __print(`Step ${i}: square=${square}, sum=${sum}`);
  }
}
function test_dict_composition() {
  __print("\n--- Test 3: Dictionary Composition ---");
  const initial_data = { "apple": 10, "banana": 5, "cherry": 20 };
  let inventory = initial_data;
  __std_dict._ext_dict__set(inventory, "date", 15);
  let count = 0;
  const fruit_list = ["apple", "banana", "cherry", "date", "elderberry"];
  for (const fruit of fruit_list) {
    if ((fruit in inventory)) {
      const val = __std_optional._ext_optional__val_or(__std_dict._ext_dict__get(inventory, fruit), 0);
      __print(`Found ${fruit}: ${val}`);
      count = (count + 1);
    } else {
      __print(`${fruit} not in inventory`);
    }
  }
  __print(`Fruits found: ${count}`);
}
test_nested_collections();
test_mutability_in_loops();
test_dict_composition();
