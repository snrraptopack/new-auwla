// Example of Result type with external JS function that throws
// This should generate try-catch wrapping in the compiled JS

@external("namespace")
type JSON<T,U> {
    @external("js", "static", "JSON", "parse")
    static fn parse(text: string): dict<T, U>?string;
}

// Define a type-safe User structure
struct User {
    name: string,
    age: number,
    email: string,
    active: bool
}

// Test with valid JSON - using array instead to avoid curly braces in string
let valid_json = "[1, 2, 3, 4, 5]";
let valid= JSON::parse(valid_json);

match valid {
    some(obj) => print("Success parsing array: {obj}"),
    none(err) => print("Error: {err}")
}

let a = Math::random();
print("Random number: {a}");

// Test with invalid JSON (will throw and be caught by try-catch)
let invalid= JSON::parse("not valid json at all");
match invalid {
    some(obj) => print("Success: {obj}"),
    none(err) => print("Error caught: {err}")
}

// Test with null
let null_test = JSON::parse("null");
match null_test {
    some(result) => print("Parsed null successfully"),
    none(err) => print("Error parsing null: {err}")
}

// Test with number
let number_test= JSON::parse("42");
match number_test {
    some(num) => print("Parsed number: {num}"),
    none(err) => print("Error: {err}")
}
