// Auwla Stress Test: Validating Extensions, Varargs, Structs, and Spreads

// 1. Extension on number (Standard Type)
extend number {
    fn add_many(self, ...others: array<number>): number {
        var res = self;
        for o in others {
            res += o;
        }
        return res;
    }

    fn is_multiple_of(self, n: number): bool {
        // Simple manual modulo calculation
        let div = Math::round_down(self / n);
        let rem = self - (div * n);
        return rem == 0;
    }
}

// 2. Extension on array (Generic Type)
extend array<number> {
    fn sum_items(self): number {
        var total = 0;
        for n in self {
            total += n;
        }
        return total;
    }

    // High-order logic in extension block
    fn filter_even_nums(self): array<number> {
        var res:number[] = [];
        for n in self {
            let div = Math::round_down(n / 2);
            let is_ev = (n - (div * 2)) == 0;
            if is_ev {
                res = [...res, n];
            }
        }
        return res;
    }
}

// 3. User-defined Struct and Extensions
struct Vector2 {
    x: number,
    y: number
}

extend Vector2 {
    fn length_sq(self): number => self.x * self.x + self.y * self.y;

    fn add_vec(self, other: Vector2): Vector2 {
        return Vector2 {
            x: self.x + other.x,
            y: self.y + other.y
        };
    }
}

// 4. Spread and Dictionary
fn test_collections() {
    let base = [1, 2, 3];
    let expanded = [0, ...base, 4, 5];
    print("Expanded Array length: {expanded.len()}");

    let scores = {"alice": 100, "bob": 80};
    let more = {...scores, "charlie": 90};
    print("Scores:");
    for (k, v) in more {
        print("{k}: {v}");
    }
}

// 5. Result and Optional handling
fn safe_divide(a: number, b: number): number?string {
    if b == 0 {
        return none("Division by zero");
    }
    return some(a / b);
}

// 6. Main execution
fn stress_main() {
    print("--- STARTING AUWLA STRESS TEST ---");

    // Varargs + Extension on standard type
    let start_val = 10;
    let final_val = start_val.add_many(1, 2, 3, 4); // 10+1+2+3+4 = 20
    print("Extended Add (10 + 1,2,3,4) = {final_val}");
    print("Is {final_val} multiple of 5? {final_val.is_multiple_of(5)}");

    // Array Extensions + Spread
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8];
    let evens = numbers.filter_even_nums();
    print("Evens count: {evens.len()}");
    print("Sum of evens: {evens.sum_items()}");

    // Structs + Custom Methods
    let v1 = Vector2 { x: 3, y: 4 };
    let v2 = Vector2 { x: 7, y: 6 };
    let v3 = v1.add_vec(v2);
    print("Vector Sum: ({v3.x}, {v3.y}), LengthSq: {v3.length_sq()}");

    // Optionals / Results
    match safe_divide(100, 5) {
        some(val) => print("100 / 5 = {val}"),
        none(err) => print("Error: {err}")
    }

    match safe_divide(99, 0) {
        some(val) => print("99 / 0 = {val}"),
        none(err) => print("Error: {err}")
    }

    // Range and Step loop
    print("Step loop (0 to 12 step 3):");
    for i in 0..13 step 3 {
        print(i);
    }

    test_collections();

    print("--- STRESS TEST COMPLETE ---");
}

stress_main();
