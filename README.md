# a casual linked list

My own linked list implemented for fun, profit and learning purposes. What would you expect?

## Installation

Since this isn't on crates.io (there's better options like the one in the standard library or Gankra's `linked-list` crate), you'd need to specify a `git` dependency in `Cargo.toml` under the `[dependencies]` section:

```toml
casual_linked_list = { git = "https://github.com/MultisampledNight/casual_linked_list.git" }
```

In addition, you're very much advised to explicitly specify a `rev` commit to check out from.

## Any questions?

### Could this be `no_std`?

Yes, _but_ it would require `alloc` for actually allocating the nodes.

### why

**Originally** I intended to implement a linked list where subslices can be reversed in _O_(1), and iteration taking _O_(_n_ + _r_), where _n_ is the list length and _r_ is the number of subslice reversals done. One reversal between two found nodes `A` and `B` would be then just adding a so-called "jump" from `A` to `B` on eachs "jump stacks", and the other way around. When iterating, the algorithm would (_\*sarcastic inhale\*_) _simply_ follow these jumps, track with a stack on which depth it is at the moment and track with another hashmap on how deep it is in each jump stack.

But this has the unfortunate side effect of the order of the reversals possibly being unpreserved, even if it would matter! For example, if one would reverse the sequence `0, 1, ..., 6` at `2..=6`, and afterwards at `0..=4`, then this would be the resulting physical representation:

```text
   original  0  1  2  3  4  5  6
      jumps  4     6     0     2
```

It's impossible to tell in this case which reversal was done first. But since reversals aren't commutative, applying `0..=4` first, then `2..=6` leads to a representation of `4 3 6 5 0 1 2`, while doing it the other way around yields `4 5 6 1 0 3 2`, which are subtly different. So, no matter how smart an iteration algorithm would be, it is _always_ wrong in some case on this representation.

"Okay", someone flusters. "Then why don't we just use a `BTreeMap` per node for the jump stack instead?"

```text
   original  0  1  2  3  4  5  6
     jump_1        6           2
     jump_2  4           0      
```

"Now this has an unambigious representation of `4 5 6 1 0 3 2`!", they shout in euphoria. Someone else raises their quiet and shy voice: "but how would you iterate in this case then? Where would you even start? For following the jump at `0` to `4`, you'd also need to know the relative direction to the target of the jump, and in this case it's even reversed since it's indirectly in the reversed range of `2..=6`. So for checking the iteration direction after a jump, you'd somehow need to know if the amount of reversal ranges under the target element is odd or even. Which requires either _even more_ bookkeeping effort, or requiring a full rescan through all (or something a bit smarter) jumps if they encapsulate the jump target, which would make iteration _at least_ _O_(_n_ + _r_<sup>2</sup>). Not to even mention about how weird the iteration depth stack would need to work, this all is—" Alright, listen up you two, yeah, I know this is possible. But I'm not sure at all if it's even worth the goal.

For now, this is a linked list. Even though it's called `ReversibleList`, it's actually just a doubly linked list. Without anything special. And maybe, someday, I'll realize that the extra bookkeeping effort would be worth it _only for changing the iteration direction of subslices cheap-ish_, and I'll come back to this and realize this all could be done way easier. But not today.

### `<insert-uncovered-question-here>`

Feel free to open an issue! owo

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
