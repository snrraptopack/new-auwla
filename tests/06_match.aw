// 06_match.aw
fn process(n: number): number?string {
    if n > 10 {
        return some(n * 2);
    } else {
        return none("Too small");
    }
}

let result1 = process(20);
match result1 {
    some(val) => print("Success: {val}"),
    none(err) => print("Error: {err}")
}

// Assignment from match
let msg = match process(5) {
    some(val) => "It worked: {val}",
    none(err) => "It failed: {err}"
};
print(msg);

