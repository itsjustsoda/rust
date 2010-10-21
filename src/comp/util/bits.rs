import std._uint;
import std._int;
import std._vec;

// FIXME: With recursive object types, we could implement binary methods like
//        union, intersection, and difference. At that point, we could write
//        an optimizing version of this module that produces a different obj
//        for the case where nbits <= 32.

// FIXME: Almost all the functions in this module should be state fns, but the
//        effect system isn't currently working correctly.

state type t = rec(vec[mutable uint] storage, uint nbits);

// FIXME: this should be a constant once they work
fn uint_bits() -> uint {
    ret 32u + ((1u << 32u) >> 27u);
}

fn create(uint nbits, bool init) -> t {
    auto elt;
    if (init) {
        elt = ~0u;
    } else {
        elt = 0u;
    }

    ret rec(storage = _vec.init_elt[mutable uint](elt, nbits / uint_bits() + 1u),
            nbits = nbits);
}

fn process(&fn(uint, uint) -> uint op, &t v0, &t v1) -> bool {
    auto len = _vec.len[mutable uint](v1.storage);

    check (_vec.len[mutable uint](v0.storage) == len);
    check (v0.nbits == v1.nbits);

    auto changed = false;

    for each (uint i in _uint.range(0u, len)) {
        auto w0 = v0.storage.(i);
        auto w1 = v1.storage.(i);

        auto w = op(w0, w1);
        if (w0 != w) {
            changed = true;
            v0.storage.(i) = w;
        }
    }

    ret changed;
}

fn lor(uint w0, uint w1) -> uint {
    ret w0 | w1;
}

fn union(&t v0, &t v1) -> bool {
    auto sub = lor;
    ret process(sub, v0, v1);
}

fn land(uint w0, uint w1) -> uint {
    ret w0 & w1;
}

fn intersect(&t v0, &t v1) -> bool {
    auto sub = land;
    ret process(sub, v0, v1);
}

fn right(uint w0, uint w1) -> uint {
    ret w1;
}

fn copy(&t v0, t v1) -> bool {
    auto sub = right;
    ret process(sub, v0, v1);
}

fn get(&t v, uint i) -> bool {
    check (i < v.nbits);

    auto bits = uint_bits();

    auto w = i / bits;
    auto b = i % bits;
    auto x = 1u & (v.storage.(w) >> b);
    ret x == 1u;
}

fn equal(&t v0, &t v1) -> bool {
    // FIXME: when we can break or return from inside an iterator loop,
    //        we can eliminate this painful while-loop
    auto len = _vec.len[mutable uint](v1.storage);
    auto i = 0u;
    while (i < len) {
        if (v0.storage.(i) != v1.storage.(i)) {
            ret false;
        }
        i = i + 1u;
    }
    ret true;
}

fn clear(&t v) {
    for each (uint i in _uint.range(0u, _vec.len[mutable uint](v.storage))) {
        v.storage.(i) = 0u;
    }
}

fn invert(&t v) {
    for each (uint i in _uint.range(0u, _vec.len[mutable uint](v.storage))) {
        v.storage.(i) = ~v.storage.(i);
    }
}

/* v0 = v0 - v1 */
fn difference(&t v0, &t v1) -> bool {
    invert(v1);
    auto b = intersect(v0, v1);
    invert(v1);
    ret b;
}

fn set(&t v, uint i, bool x) {
    check (i < v.nbits);

    auto bits = uint_bits();

    auto w = i / bits;
    auto b = i % bits;
    auto w0 = v.storage.(w);
    auto flag = 1u << b;
    if (x) {
        v.storage.(w) = v.storage.(w) | flag;
    } else {
        v.storage.(w) = v.storage.(w) & ~flag;
    }
}

fn init_to_vec(&t v, uint i) -> uint {
    if (get(v, i)) {
        ret 1u;
    } else {
        ret 0u;
    }
}

fn to_vec(&t v) -> vec[uint] {
    auto sub = bind init_to_vec(v, _);
    ret _vec.init_fn[uint](sub, v.nbits);
}

// FIXME: can we just use structural equality on to_vec?
fn eq_vec(&t v0, &vec[uint] v1) -> bool {
    check (v0.nbits == _vec.len[uint](v1));
    auto len = v0.nbits;
    auto i = 0u;
    while (i < len) {
        auto w0 = get(v0, i);
        auto w1 = v1.(i);
        if ((!w0 && w1 != 0u) || (w0 && w1 == 0u)) {
            ret false;
        }
        i = i + 1u;
    }
    ret true;
}

fn test() {
    auto act;
    auto exp;

    // -----------------------------------------------------------------------
    // Tests of 0-element bit-vectors.

    act = create(0u, false);
    exp = _vec.init_elt[uint](0u, 0u);
    // FIXME: why can't I write vec[uint]()?
    check (eq_vec(act, exp));

    // -----------------------------------------------------------------------
    // Tests of 1-element bit-vectors.

    act = create(1u, false);
    check (eq_vec(act, vec(0u)));

    act = create(1u, true);
    check (eq_vec(act, vec(1u)));

    // -----------------------------------------------------------------------
    // Tests of 10-element bit-vectors.

    // all 0
    act = create(10u, false);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u)));

    // all 1
    act = create(10u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u)));

    // mixed
    act = create(10u, false);
    set(act, 0u, true);
    set(act, 1u, true);
    set(act, 2u, true);
    set(act, 3u, true);
    set(act, 4u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 0u, 0u, 0u, 0u, 0u)));

    // mixed
    act = create(10u, false);
    set(act, 5u, true);
    set(act, 6u, true);
    set(act, 7u, true);
    set(act, 8u, true);
    set(act, 9u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 1u, 1u, 1u, 1u, 1u)));

    // mixed
    act = create(10u, false);
    set(act, 0u, true);
    set(act, 3u, true);
    set(act, 6u, true);
    set(act, 9u, true);
    check (eq_vec(act, vec(1u, 0u, 0u, 1u, 0u, 0u, 1u, 0u, 0u, 1u)));

    // -----------------------------------------------------------------------
    // Tests of 31-element bit-vectors.

    // all 0
    act = create(31u, false);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u)));

    // all 1
    act = create(31u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u)));

    // mixed
    act = create(31u, false);
    set(act, 0u, true);
    set(act, 1u, true);
    set(act, 2u, true);
    set(act, 3u, true);
    set(act, 4u, true);
    set(act, 5u, true);
    set(act, 6u, true);
    set(act, 7u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u)));

    // mixed
    act = create(31u, false);
    set(act, 16u, true);
    set(act, 17u, true);
    set(act, 18u, true);
    set(act, 19u, true);
    set(act, 20u, true);
    set(act, 21u, true);
    set(act, 22u, true);
    set(act, 23u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u)));

    // mixed
    act = create(31u, false);
    set(act, 24u, true);
    set(act, 25u, true);
    set(act, 26u, true);
    set(act, 27u, true);
    set(act, 28u, true);
    set(act, 29u, true);
    set(act, 30u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u)));

    // mixed
    act = create(31u, false);
    set(act, 3u, true);
    set(act, 17u, true);
    set(act, 30u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 1u)));

    // -----------------------------------------------------------------------
    // Tests of 32-element bit-vectors.

    // all 0
    act = create(32u, false);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u)));

    // all 1
    act = create(32u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u)));

    // mixed
    act = create(32u, false);
    set(act, 0u, true);
    set(act, 1u, true);
    set(act, 2u, true);
    set(act, 3u, true);
    set(act, 4u, true);
    set(act, 5u, true);
    set(act, 6u, true);
    set(act, 7u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u)));

    // mixed
    act = create(32u, false);
    set(act, 16u, true);
    set(act, 17u, true);
    set(act, 18u, true);
    set(act, 19u, true);
    set(act, 20u, true);
    set(act, 21u, true);
    set(act, 22u, true);
    set(act, 23u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u)));

    // mixed
    act = create(32u, false);
    set(act, 24u, true);
    set(act, 25u, true);
    set(act, 26u, true);
    set(act, 27u, true);
    set(act, 28u, true);
    set(act, 29u, true);
    set(act, 30u, true);
    set(act, 31u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u)));

    // mixed
    act = create(32u, false);
    set(act, 3u, true);
    set(act, 17u, true);
    set(act, 30u, true);
    set(act, 31u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 1u, 1u)));

    // -----------------------------------------------------------------------
    // Tests of 33-element bit-vectors.

    // all 0
    act = create(33u, false);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u)));

    // all 1
    act = create(33u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           1u)));

    // mixed
    act = create(33u, false);
    set(act, 0u, true);
    set(act, 1u, true);
    set(act, 2u, true);
    set(act, 3u, true);
    set(act, 4u, true);
    set(act, 5u, true);
    set(act, 6u, true);
    set(act, 7u, true);
    check (eq_vec(act, vec(1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u)));

    // mixed
    act = create(33u, false);
    set(act, 16u, true);
    set(act, 17u, true);
    set(act, 18u, true);
    set(act, 19u, true);
    set(act, 20u, true);
    set(act, 21u, true);
    set(act, 22u, true);
    set(act, 23u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u)));

    // mixed
    act = create(33u, false);
    set(act, 24u, true);
    set(act, 25u, true);
    set(act, 26u, true);
    set(act, 27u, true);
    set(act, 28u, true);
    set(act, 29u, true);
    set(act, 30u, true);
    set(act, 31u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                           0u)));

    // mixed
    act = create(33u, false);
    set(act, 3u, true);
    set(act, 17u, true);
    set(act, 30u, true);
    set(act, 31u, true);
    set(act, 32u, true);
    check (eq_vec(act, vec(0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u,
                           0u, 0u, 0u, 0u, 0u, 0u, 1u, 1u,
                           1u)));
}

//
// Local Variables:
// mode: rust
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// compile-command: "make -k -C ../.. 2>&1 | sed -e 's/\\/x\\//x:\\//g'";
// End:
//
