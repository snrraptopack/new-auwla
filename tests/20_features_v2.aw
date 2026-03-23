// Test file for Spread Operator and For-loop Step

// 1. Array Spread
let base_arr = [2, 3];
let spread_arr = [1, ...base_arr, 4];
print("Spread Array: ");
for x in spread_arr {
    print(x);
}

// 2. Dict Spread
let base_dict = {"a": 2, "c": 3};
let spread_dict = {"a": 1, ...base_dict, "d": 4};
print("Spread Dict keys (approx):");
// Note: iteration might not be ordered
// for k, v in spread_dict { ... } // assuming this syntax exists or similar

// 3. Varargs Function
fn sum_all(...nums: array<number>): number {
    var  total = 0;
    for n in nums {
        total += n;
    }
    return total;
}

let s = sum_all(1, 2, 3, 4, 5);
print("Sum All (1..5):");
print(s);

// 4. For-loop Step
print("Step loop (0 to 10 step 2):");
for i in 0..11 step 2 {
    print(i);
}

// 5. Method Varargs
extend number {
    fn multi_add(self, ...others: array<number>): number {
        var res = self;
        for o in others {
            res += o;
        }
        return res;
    }
}

let val = 10;
let val2 = val.multi_add(1, 1, 1);
print("Method multi_add (10 + 1 + 1 + 1):");
print(val2);
