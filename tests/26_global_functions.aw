@external("js", "function", "__print")
fn print(message: string): void;

@external("js", "static", "Math", "max")
fn max2(a: number, b: number): number;

@external("js", "method", "toUpperCase")
fn to_upper(s: string): string;

@external("js", "property", "length")
fn strlen(s: string): number;

fn tag(msg: string): string => "[auwla] " + msg;

fn main() {
    print(tag("global fn test"));

    let n = max2(3, 11);
    print("max2 = {n}");

    let loud = to_upper("auwla");
    print("loud = {loud}");

    let l = strlen("hello");
    print("strlen = {l}");
}

main();
