// Testing tuple support

// Basic tuple literals
let point: (number, number) = (10, 20);
let person: (string, number, bool) = ("Alice", 30, true);

// Tuple destructuring
let (x, y) = point;
print("Point coordinates: x={x}, y={y}");

let (name, age, active) = person;
print("Person: {name}, age {age}, active: {active}");

// Function returning tuple
fn get_user(): (string, number) {
    return ("Bob", 25);
}

let (user_name, user_age) = get_user();
print("User: {user_name}, age {user_age}");

// Nested tuples
let nested: ((number, number), (number, number)) = ((1, 2), (3, 4));
let ((a, b), (c, d)) = nested;
print("Nested: a={a}, b={b}, c={c}, d={d}");

// Pattern matching with tuples
fn describe_point(p: (number, number)) {
    match p {
        (0, 0) => print("Origin"),
        (x, 0) => print("On X-axis at {x}"),
        (0, y) => print("On Y-axis at {y}"),
        (x, y) => print("Point at ({x}, {y})")
    }
}

describe_point((0, 0));
describe_point((5, 0));
describe_point((0, 3));
describe_point((10, 20));

// Mixed types in tuple
let mixed: (string, number, bool, string) = ("test", 42, false, "end");
let (s1, n, b, s2) = mixed;
print("Mixed: {s1}, {n}, {b}, {s2}");
