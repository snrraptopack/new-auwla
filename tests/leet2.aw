// tests/leet2.aw — Advanced composition and mutability test

// Test 1: Nested Collections and Mutation
fn test_nested_collections() {
    print("--- Test 1: Nested Collections ---");
    var registry: dict<string, number[]> = {};
    registry.set("primes", [2, 3, 5]);
    registry.set("fib", [1, 1, 2, 3]);

    print("registry-primes",registry.get("primes"));
    // Access and mutate
    if "primes" in registry {
        let p = registry.get("primes");
        print("Primes: {p}");
    }

    var total_elements = 0;
    let keys = ["primes", "fib"];
    for key in keys {
        if key in registry {
            total_elements = total_elements +( registry.get(key).val_or([])).len();
        }
    }
    print("Total elements: {total_elements}");
}

// Test 2: Var vs Let in Loops
fn test_mutability_in_loops() {
    print("\n--- Test 2: Mutability in Loops ---");
    var sum = 0;
    for i in 1..5 {
        let square = i * i; // immutable
        var running_avg = square + sum; // mutable
        if i % 2 == 0 {
            running_avg = running_avg / 2;
        }
        sum = sum + running_avg;
        print("Step {i}: square={square}, sum={sum}");
    }
}

// Test 3: Complex Dictionary Operations
fn test_dict_composition() {
    print("\n--- Test 3: Dictionary Composition ---");
    let initial_data = {
        "apple": 10,
        "banana": 5,
        "cherry": 20
    };

    var inventory = initial_data;
    inventory.set("date", 15);

    var count = 0;
    let fruit_list = ["apple", "banana", "cherry", "date", "elderberry"];

    for fruit in fruit_list {
        if fruit in inventory {
            let val = inventory.get(fruit).val_or(0);
            print("Found {fruit}: {val}");
            count = count + 1;
        } else {
            print("{fruit} not in inventory");
        }
    }
    print("Fruits found: {count}");
}

// Run tests
test_nested_collections();
test_mutability_in_loops();
test_dict_composition();
