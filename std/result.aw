// std/result.aw — Methods for T?E types

extend T?E {
    fn val_or(self, default_v: T): T {
        return match self {
            some(v) => v,
            none(_) => default_v,
        };

    }

    fn is_ok(self): bool {
        return match self {
            some(_) => true,
            none(_) => false,
        };

    }

    fn is_err(self): bool {
        return match self {
            some(_) => false,
            none(_) => true,
        };
    }

    //TODO: Fix typechecker to recognize E type parameter
    fn get_err(self): E? {
        return match self {
            some(_) => none,
            none(err) => some(err),
        };
    }
}
