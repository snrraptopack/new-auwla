// 24_dict_methods.aw — Testing dictionary extension methods

print("=== Basic Operations ===");
let scores: dict<string, number> = {
    "Alice": 95,
    "Bob": 87,
    "Charlie": 92
};

print("Length: {scores.len()}");
print("Is empty: {scores.is_empty()}");
print("Contains Alice: {scores.contains("Alice")}");
print("Contains David: {scores.contains("David")}");

// Safe get
let alice_score = scores.get("Alice");
print("Alice's score: {alice_score}");
let david_score = scores.get("David");
print("David's score: {david_score}");

// Set new value
scores.set("David", 88);
print("After adding David, length: {scores.len()}");

print("\n=== Iteration ===");
let keys = scores.keys();
print("Keys: {keys}");

let values = scores.values();
print("Values: {values}");

print("\n=== Transformation ===");
// Map: double all scores
let double_score = (score: number): number => score * 2;
let doubled = scores.map(double_score);
print("Doubled scores: {doubled}");

// Filter: only scores >= 90
let is_high_score = (name: string, score: number): bool => score >= 90;
let high_scores = scores.filter(is_high_score);
print("High scores (>=90): {high_scores}");

print("\n=== Merging ===");
let extra_scores: dict<string, number> = {
    "Eve": 91,
    "Frank": 85
};

let all_scores = scores.merge(extra_scores);
print("Merged scores length: {all_scores.len()}");

// Using operator +
let combined = scores + extra_scores;
print("Combined with + length: {combined.len()}");

print("\n=== Pick & Omit ===");
let selected = all_scores.pick(["Alice", "Eve", "Frank"]);
print("Picked Alice, Eve, Frank: {selected}");

let without_bob = all_scores.omit(["Bob", "David"]);
print("Omitted Bob, David: {without_bob}");

print("\n=== Querying ===");
// Find key where value > 93
let is_top_score = (score: number): bool => score > 93;
let top_scorer = all_scores.find_key(is_top_score);
print("Top scorer (>93): {top_scorer}");

// Any score > 90?
let is_above_90 = (score: number): bool => score > 90;
let has_high = all_scores.any(is_above_90);
print("Any score > 90: {has_high}");

// All scores >= 80?
let is_passing = (score: number): bool => score >= 80;
let all_passing = all_scores.every(is_passing);
print("All scores >= 80: {all_passing}");

// Count scores >= 90
let count_high = (name: string, score: number): bool => score >= 90;
let high_count = all_scores.count(count_high);
print("Count of scores >= 90: {high_count}");

print("\n=== For Each ===");
print("Printing all scores:");
let print_score = (name: string, score: number): void => {
    print("  {name}: {score}");
};
all_scores.for_each(print_score);

print("\n=== Clear & Remove ===");
let temp: dict<string, number> = { "x": 1, "y": 2, "z": 3 };
print("Before operations - length: {temp.len()}");
let removed = temp.remove("y");
print("Removed 'y': {removed}, length now: {temp.len()}");
temp.clear();
print("After clear - length: {temp.len()}, is empty: {temp.is_empty()}");
