// std/optional.aw — Methods for T? types
extend T? {
    fn val_or(self, default_v: T): T {
        return match self {
            some(v) => v,
            none => default_v,
        };
    }

    fn is_some(self): bool {
        return match self {
            some(_) => true,
            none => false,
        };

    }

    fn is_none(self): bool {
        return match self {
            some(_) => false,
            none => true,
        };
    }
}
