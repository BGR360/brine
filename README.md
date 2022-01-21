# Brine

Brine is my attempt at writing a Minecraft client in Rust using the [Bevy game
engine](https://bevyengine.org/).

The thing that makes Brine unique is that it is based on a high-level
abstraction of the Minecraft protocol that isn't specific to any one game
version[^1]. This is defined in the [`brine_proto`](crates/brine_proto) crate.

The goal for this API is to be generic enough to express the logic of the
Minecraft game for any version of the game, including (in theory) Bedrock
Edition. We'll see if that ends up being possible, I still have a lot to learn
about the differences in Minecraft versions.

See [`crates/README.md`](crates/README.md) for a more detailed overview of the
architecture.

[^1]: Being "protocol-generic" isn't new; see projects that came before like
[steven](https://github.com/thinkofname/steven)/[stevenarella](https://github.com/iceiix/stevenarella).
What is new (to my knowledge) is the attempt to abstract the logic of Minecraft
into an API that can be used to support any version without significant (or any)
modification to the client's game logic.

I don't recall what exactly gave me the idea for the name "Brine." I think I was
thinking of Rust, then crustaceans, then the sea, and I landed on "Brine." I
only later realized that it was short for
[Herobrine](https://minecraft.fandom.com/wiki/Herobrine) :-).
