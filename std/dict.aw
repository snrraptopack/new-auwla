// std/dict.aw — Standard library dictionary extensions

extend dict<K, V> {
    // --- JS interop: properties ---
    @external("js", "property", "size")
    fn len(self): number;

    // --- JS interop: methods ---
    @external("js", "method", "has")
    fn contains(self, key: K): bool;

    @external("js", "method", "delete")
    fn remove(self, key: K): bool;

    @external("js", "method", "clear")
    fn clear(self): void;

    // --- Safe Auwla methods ---
    fn get(self, key: K): V? {
        if self.contains(key) {
            return some(self[key]); // Internal access allowed
        }
        return none;
    }

    fn set(self, key: K, value: V): void {
        self[key] = value;
    }

    fn is_empty(self): bool => self.len() == 0;
}
