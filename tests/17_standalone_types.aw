// Test standalone namespace and class types

// ─── Namespace: Math ───
@external("namespace")
type Math {
    @external("js", "method", "floor")
    static fn round_down(x: number): number;

    @external("js", "method", "ceil")
    static fn round_up(x: number): number;

    @external("js", "method", "random")
    static fn random(): number;

    @external("js", "property", "PI")
    static fn pi(): number;
}

// ─── Namespace: console ───
@external("namespace")
type Console {
    @external("js", "method", "log")
    static fn log(msg: string): void;
}

// ─── Class: WebSocket (constructor + instance methods) ───
// @external("class")
// type WebSocket {
//     @external("js", "constructor")
//     static fn create(url: string): Self;

//     @external("js", "property", "readyState")
//     fn ready_state(self): number;

//     @external("js", "method", "send")
//     fn send(self, data: string): void;

//     @external("js", "method", "close")
//     fn close(self): void;
// }

fn main() {
    // Test namespace static method calls
    let x = Math::round_down(3.7);
    print("floor(3.7) = {x}");

    let y = Math::round_up(3.2);
    print("ceil(3.2) = {y}");

    let r = Math::random();
    print("random = {r}");

    // Test namespace static property
    let pi = Math::pi();
    print("PI = {pi}");

    // Test another namespace
    Console::log("hello from Console::log");
}

main();
