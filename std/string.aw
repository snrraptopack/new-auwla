// std/string.aw — Standard library string extensions

extend string {
    // --- JS interop: property ---
    @external("js", "property", "length")
    fn len(self): number;

    // --- JS interop: methods ---
    @external("js", "method", "charAt")
    fn char_at(self, index: number): string;

    @external("js", "method", "indexOf")
    fn index_of(self, search: string): number;

    @external("js", "method", "repeat")
    fn repeat(self, times: number): string;

    @external("js", "method", "at")
    fn get(self, index: number): char?;

    @external("js", "method", "toUpperCase")
    fn to_upper(self): string;

    @external("js", "method", "toLowerCase")
    fn to_lower(self): string;

    @external("js", "method", "trim")
    fn trim(self): string;

    @external("js", "method", "startsWith")
    fn starts_with(self, prefix: string): bool;

    @external("js", "method", "endsWith")
    fn ends_with(self, suffix: string): bool;

    @external("js", "method", "includes")
    fn contains(self, search: string): bool;

    @external("js", "method", "slice")
    fn slice(self, start: number, end: number): string;

    @external("js", "method", "split")
    fn split(self, delimiter: string): string[];

    @external("js", "method", "replace")
    fn replace(self, search: string, replacement: string): string;

    @external("js", "method", "trimStart")
    fn trim_start(self): string;

    @external("js", "method", "trimEnd")
    fn trim_end(self): string;

    @external("js", "method", "padStart")
    fn pad_start(self, length: number, fill: string): string;

    @external("js", "method", "padEnd")
    fn pad_end(self, length: number, fill: string): string;

    @external("js", "method", "replaceAll")
    fn replace_all(self, search: string, replacement: string): string;

    @external("js", "method", "lastIndexOf")
    fn last_index_of(self, search: string): number;

    @external("js", "method", "normalize")
    fn normalize(self): string;

    @external("js", "method", "codePointAt")
    fn code_at(self, index: number): number?;

    @external("js", "static", "String", "fromCodePoint")
    static fn from_code(code: number): string;

}

// --- Operator extensions ---
extend string {
     operator *(self, other: number): string {
        return self.repeat(other);
    }
}

// --- Pure Auwla methods ---
extend string{

    fn shout(self): string => self + "!!!";
    fn whisper(self): string => self + "...";

    fn first_n(self, n: number): string {
        var result = "";
        for i in 0 ..< n {
            result = result + self.char_at(i);
        }
        return result;
    }

    fn is_empty(self): bool => self.len() == 0;

    fn reverse(self): string {
        var result = "";
        for i in 0 ..< self.len() {
            result = self.char_at(self.len() - 1 - i) + result;
        }
        return result;
    }

}
