// test enum declarations, exhaustive matching, and argument bindings

enum Status {
    Active,
    Inactive,
    Banned(string) // reason
}

let ok_status = Status::Active;
let ok_inactive = Status::Inactive;
let banned_status = Status::Banned("Violation of terms");

print("--- EXHAUSTIVE MATCH TESTING ---");

fn check_status(status: Status) {
    match status {
        Active => {
            print("Status is Active");
        },
        Inactive => {
            print("Status is Inactive");
        },
        Banned(reason) => {
            print("Status is Banned: ");
            print(reason);
        }
    }
}

check_status(ok_status);
check_status(ok_inactive);
check_status(banned_status);

print("--- DIRECT MATCH ASSIGNMENT ---");

let message = match banned_status {
    Active => "All good",
    Inactive => "User is inactive",
    Banned(reason) => reason
};

print(message);
