// 07_loops.aw

// Testing while loops
var i = 0;
while i < 3 {
    print("While Loop i: {i}");
    i = i + 1;
}

// Testing for-in loops with arrays
let items = ["apple", "banana", "cherry"];
for item in items {
    print("For Loop item: {item}");
}

// Testing for-in loops with ranges
for num in 5..7 {
    print("For Loop range: {num}");
}

let one = 10