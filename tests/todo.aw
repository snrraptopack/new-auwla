enum TaskStatus {
    Pending,
    Done,
    Archived
}

struct Task {
    id: number,
    title: string,
    status: TaskStatus
}

// We extend the array specifically for Task logic
extend array<Task> {
    fn find_one(self, id: number): Task?string {
        for t in self {
            if t.id == id {
                return some(t);
            }
        }
        return none("id not found");
    }

    fn print_summary(self): void {
        print("Summary of {self.len()} tasks:"); // Tests existing len() extension
        for t in self {
            let status_icon = match t.status {
                Pending => "⏳",
                Done => "✅",
                Archived => "📦",
            };
            print("{status_icon} {t.title}");
        }
    }
}

fn main() {
    let my_tasks = [
        Task { id: 101, title: "Refactor Codegen", status: TaskStatus::Done },
        Task { id: 102, title: "Secure JS Interop", status: TaskStatus::Pending },
        Task { id: 103, title: "Ship Auwla", status: TaskStatus::Pending }
    ];

    print("--- AUWLA TASK MANAGER ---");
    my_tasks.print_summary();

    // Testing the Match Block Emission fix (multi-line logic in arms)
    print("Searching for Task 102...");
    match my_tasks.find_one(1022) {
        some(t) => {
            print("Target Found!");
            print("ID: {t.id}");
            print("Label: {t.title}");
            if t.id > 100 {
                print("Priority: High (Legacy System)");
            }
        },
        none(msg) => {
            print("Error: {msg}");
        },
    }

    // Testing safe 'char?' wrapping on a literal string
    let version = "v1.0.0";
    print("Checking version prefix...");
    match version.get(0) {
        some(c) => {
            if c == 'v' {
                print("Version starts with 'v' - Valid.");
            } else {
                print("Unknown version format: {c}");
            }
        },
        none => print("Empty version string detected.")
    }
}

let ama = 1 .. 10;

main();


