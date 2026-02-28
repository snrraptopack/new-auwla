// 03_ranges.aw
for x in 1..5 {
    print("Inclusive: {x}");
}

for y in 1..<5 {
    print("Exclusive: {y}");
}

// Char ranges
for c in 'a'..'d' {
    print("Char: {c}");
}

let a = 'a' .. 'z';
print("Array from range: {a}");