use rand::seq::IndexedRandom;

pub fn generate_name() -> String {
  let mut rng = rand::rng();
  let adj = ADJECTIVES.choose(&mut rng).unwrap();
  let noun = NOUNS.choose(&mut rng).unwrap();
  format!("{adj}-{noun}")
}

const ADJECTIVES: &[&str] = &[
  "abandoned", "abstract", "acoustic", "aged", "allocated", "ancient", "angry", "atomic",
  "balanced", "bitter", "blazing", "bold", "broken", "burning", "calm", "careful", "chaotic",
  "clever", "cold", "cosmic", "crispy", "crystal", "curious", "dancing", "dark", "dazzling",
  "deep", "digital", "distant", "dusty", "eager", "early", "electric", "elegant", "empty",
  "endless", "epic", "eternal", "fading", "fallen", "fancy", "fast", "fierce", "flaky", "flying",
  "foggy", "forked", "frozen", "gentle", "gilded", "global", "golden", "grand", "grumpy", "half",
  "happy", "harsh", "hasty", "heavy", "hidden", "hollow", "humble", "icy", "idle", "inner",
  "iron", "jagged", "jolly", "keen", "kind", "lazy", "light", "lively", "lonely", "lost", "loud",
  "lucky", "lunar", "mellow", "merged", "mighty", "misty", "modern", "molten", "muted", "narrow",
  "nested", "noble", "odd", "old", "open", "orange", "outer", "packed", "pale", "plain", "polar",
  "polite", "proud", "pure", "quiet", "rapid", "raw", "rebel", "red", "rising", "rocky", "rough",
  "round", "royal", "rugged", "rustic", "rusty", "salty", "savage", "scarce", "secret", "serene",
  "sharp", "shiny", "silent", "silver", "simple", "sleepy", "slim", "slow", "smoky", "snowy",
  "solar", "solid", "sparse", "spicy", "static", "steady", "steep", "still", "stormy", "strict",
  "strong", "subtle", "sudden", "sunny", "super", "swift", "tall", "tame", "thick", "thin",
  "tidy", "tiny", "tough", "twisted", "vague", "vivid", "warm", "wary", "weird", "wild", "wired",
  "wise", "worn", "young", "zealous",
];

const NOUNS: &[&str] = &[
  "acorn", "anchor", "ant", "apple", "arrow", "axe", "badger", "banana", "barrel", "basket",
  "beacon", "bear", "beetle", "bell", "berry", "bison", "blade", "bloom", "bobcat", "bolt",
  "bone", "branch", "breeze", "brick", "bridge", "brook", "brush", "bucket", "bulb", "bunny",
  "cactus", "candle", "canyon", "carpet", "cedar", "chalk", "cherry", "cliff", "clover", "cobra",
  "comet", "coral", "crane", "creek", "crown", "cube", "dagger", "dawn", "deer", "delta", "dingo",
  "dock", "dome", "donkey", "dove", "dragon", "drum", "duck", "eagle", "ember", "falcon", "fern",
  "ferret", "finch", "flame", "flask", "flint", "forest", "fossil", "fox", "frog", "gale",
  "garden", "gecko", "ghost", "glacier", "goat", "goose", "grape", "grove", "hammer", "harbor",
  "hare", "hawk", "hedge", "heron", "hill", "honey", "hornet", "ice", "igloo", "iris", "island",
  "ivy", "jackal", "jade", "jaguar", "jay", "jungle", "kelpie", "kernel", "kettle", "kite",
  "knot", "lake", "lark", "latch", "leaf", "lemon", "lily", "lion", "lotus", "lynx", "maple",
  "marsh", "meadow", "meteor", "mint", "mole", "moose", "moth", "mouse", "newt", "night", "oak",
  "ocean", "olive", "orca", "orchid", "osprey", "otter", "owl", "panda", "panther", "parrot",
  "peach", "pebble", "pelican", "pepper", "pine", "plum", "pond", "quail", "quartz", "rabbit",
  "rain", "raven", "reef", "ridge", "river", "robin", "rock", "sage", "salmon", "seal", "seed",
  "shell", "silk", "slate", "snail", "snake", "spark", "sphinx", "spider", "sprout", "squid",
  "star", "stone", "storm", "stork", "stream", "summit", "swan", "thorn", "tiger", "torch",
  "trout", "tulip", "turtle", "valley", "vine", "violet", "viper", "walrus", "wave", "whale",
  "willow", "wolf", "wren", "yak", "zebra",
];
