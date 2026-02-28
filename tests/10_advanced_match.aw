// Test advanced match expressions
// Combining literals, OR logic, and wildcards, plus enums!

fn check_status(code: number):string {
    match code {
        200 | 201 => "Success",
        400 | 401 | 403 | 404 => "Client Error",
        500..599 => "Server Error", // We'll test ranges later, just literals for now
        _ => "Unknown Error"
    }
}

// Let's rewrite the above to use the features we just built
fn check_status_code(code: number):string {
    match code {
        200 | 201 => "Success",
        400 | 401 | 403 | 404 => "Client Error",
        500..599 => "Server Error",
        _ => "Unknown Error"
    }
}

print(check_status_code(200));
print(check_status_code(404));
print(check_status_code(503));
print(check_status_code(999));

fn handle_event(event: string) {
    match event {
        "click" | "tap" | "keypress" => print("User interaction detected!"),
        "load" | "unload" => print("Page lifecycle event!"),
        _ => print("Unknown event!")
    }
}

handle_event("click");
handle_event("load");
handle_event("hover");

// Enum advanced matching
enum AppState {
    Loading,
    Ready,
    Error(string),
    Offline
}

fn handle_state(state: AppState) {
    match state {
        Loading | Offline => print("App is not interactive right now."),
        Ready => print("App is ready!"),
        Error(msg) => print("App encountered an error: {msg}")
    }
}

handle_state(AppState::Loading);
handle_state(AppState::Ready);
handle_state(AppState::Offline);
handle_state(AppState::Error("Network timeout"));

// Test match guards and variable bindings
fn check_retries(current_retries: number, max_retries: number) {
    match current_retries {
        count if count >= max_retries => print("Failed: Too many retries!"),
        count => print("Retrying... attempt {count}")
    }
}

check_retries(4, 3);
check_retries(1, 3);
