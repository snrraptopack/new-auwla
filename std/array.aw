// std/array.aw — Standard library array extensions

extend array<T> {
    // --- JS interop: properties ---
    @external("js", "property", "length")
    fn len(self): number;

    // --- JS interop: methods ---
    @external("js", "method", "push")
    fn push(self, val: T): void;

    @external("js", "method", "pop")
    fn pop(self): T?;

    @external("js", "method", "reverse")
    fn reverse(self): T[];

    @external("js", "method", "join")
    fn join(self, separator: string): string;

    @external("js", "method", "includes")
    fn contains(self, val: T): bool;

    @external("js", "method", "indexOf")
    fn index_of(self, val: T): number;

    @external("js", "method", "slice")
    fn slice(self, start: number, end: number): T[];

    @external("js", "method", "map")
    fn map(self, f: (T) => T): T[];

    @external("js", "method", "filter")
    fn filter(self, predicate: (T) => bool): T[];

    @external("js", "method", "forEach")
    fn for_each(self, f: (T) => void): void;

    @external("js", "method", "find")
    fn find(self, predicate: (T) => bool): T?;

    @external("js", "method", "every")
    fn every(self, predicate: (T) => bool): bool;

    @external("js", "method", "some")
    fn any(self, predicate: (T) => bool): bool;

    @external("js", "method", "flat")
    fn flat(self): T[];

    @external("js", "method", "sort")
    fn sort(self): T[];

    @external("js", "method", "concat")
    fn concat(self, other: T[]): T[];

    @external("js", "static", "Array", "isArray")
    static fn is_array(val: T[]): bool;

    // --- Pure Auwla methods (no JS equivalent) ---
    fn low(self): number => 0;
    fn high(self): number => self.len();

    fn last(self): T? {
        if self.len() > 0 {
            return some(self[self.len() - 1]);
        }
        return none;
    }

    fn first(self): T? {
        if self.len() > 0 {
            return some(self[0]);
        }
        return none;
    }

    fn is_empty(self): bool => self.len() == 0;

    fn shuffle(self) {
        for i in 0 ..< self.len() {
            let random = Math::round_down(Math::random() * self.len());
            let temp = self[i];
            self[i] = self[random];
            self[random] = temp;
        }
    }
}

extend array<number> {
    @external("js", "method", "reduce")
    fn reduce(self, f: (number, number) => number, initial: number): number;

    fn sum(self): number {
        return self.reduce((acc: number, val: number) => acc + val, 0);
    }

    fn max(self): number {
        var c_max = self[0];
        for i in 1 ..< self.len() {
            if self[i] > c_max {
                c_max = self[i];
            }
        }
        return c_max;
    }

    fn min(self): number {
        var c_min = self[0];
        for i in 1 ..< self.len() {
            if self[i] < c_min {
                c_min = self[i];
            }
        }
        return c_min;
    }
}
