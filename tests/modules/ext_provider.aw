// Provides extensions for testing cross-file visibility
export extend array<number> {
    fn sum(self): number {
        var total = 0;
        for x in self {
            total = total + x;
        }
        return total;
    }
}
