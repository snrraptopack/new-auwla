let nums = [1, 2, 3, 4, 5];

// Map uses 'x' without type annotations
let doubled = nums.map((x) => x * 2);

// Filter uses 'x' without type annotations
let greater_than_two = nums.filter((x) => x > 2);

// Reduce uses 'acc' and 'val' without type annotations
let sum = nums.reduce((acc, val) => acc + val, 0);

print("Doubled:", doubled);
print("GreaterThanTwo:", greater_than_two);
print("Sum:", sum);
