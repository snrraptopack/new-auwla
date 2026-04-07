// std/dict.aw — Standard library dictionary extensions

extend <K, V> dict<K, V> {
    // --- External static helpers ---
    @external("js", "static", "Reflect", "deleteProperty")
    static fn delete_property(obj: dict<K, V>, key: K): bool;

    @external("js", "static", "Object", "keys")
    static fn object_keys(obj: dict<K, V>): string[];

    // --- Basic operations ---

    fn len(self): number {
        let keys = dict::object_keys(self);
        var count = 0;
        for _ in keys {
            count = count + 1;
        }
        return count;
    }

    fn contains(self, key: K): bool {
        return key in self;
    }

    fn remove(self, key: K): bool {
        if key in self {
            return dict::delete_property(self, key);
        }
        return false;
    }

    fn clear(self): void {
        let keys = dict::object_keys(self);
        for key in keys {
            dict::delete_property(self, key);
        }
    }

    fn is_empty(self): bool => self.len() == 0;

    // Safe get with optional return
    fn get(self, key: K): V? {
        if key in self {
            return some(self[key]);
        }
        return none;
    }

    fn set(self, key: K, value: V): void {
        self[key] = value;
    }

    // --- Iteration & Transformation ---

    fn keys(self): K[] {
        var result: K[] = [];
        for (k, _) in self {
            result.push(k);
        }
        return result;
    }

    fn values(self): V[] {
        var result: V[] = [];
        for (_, v) in self {
            result.push(v);
        }
        return result;
    }

    fn map<U>(self, f: (V) => U): dict<K, U> {
        var result: dict<K, U> = {};
        for (k, v) in self {
            result[k] = f(v);
        }
        return result;
    }

    fn filter(self, predicate: (K, V) => bool): dict<K, V> {
        var result: dict<K, V> = {};
        for (k, v) in self {
            if predicate(k, v) {
                result[k] = v;
            }
        }
        return result;
    }

    fn for_each(self, f: (K, V) => void): void {
        for (k, v) in self {
            f(k, v);
        }
    }

    // --- Merging & Combining ---

    fn merge(self, other: dict<K, V>): dict<K, V> {
        var result: dict<K, V> = {};
        for (k, v) in self {
            result[k] = v;
        }
        for (k, v) in other {
            result[k] = v;
        }
        return result;
    }

    operator +(self, other: dict<K, V>): dict<K, V> {
        return self.merge(other);
    }

   fn pick(self, keys: K[]): dict<K, V> {
        var result: dict<K, V> = {};
        for key in keys {
            if key in self {
                result[key] = self[key]; // Direct mapping
            }
        }
        return result;
    }

    fn omit(self, keys: K[]): dict<K, V> {
        // 1. Copy the whole dictionary
        var result: dict<K, V> = {};
        for (k, v) in self {
            result[k] = v;
        }
        
        // 2. Delete unwanted keys (use static method since std has no typecheck info)
        for key in keys {
            dict::delete_property(result, key);
        }
        
        return result;
    }

    // --- Querying & Searching ---
    fn find_key(self, predicate: (V) => bool): K? {
        for (k, v) in self {
            if predicate(v) {
                return some(k);
            }
        }
        return none;
    }

    fn any(self, predicate: (V) => bool): bool {
        for (_, v) in self {
            if predicate(v) {
                return true;
            }
        }
        return false;
    }

    fn every(self, predicate: (V) => bool): bool {
        for (_, v) in self {
            if !predicate(v) {
                return false;
            }
        }
        return true;
    }

    fn count(self, predicate: (K, V) => bool): number {
        var count = 0;
        for (k, v) in self {
            if predicate(k, v) {
                count = count + 1;
            }
        }
        return count;
    }
}
