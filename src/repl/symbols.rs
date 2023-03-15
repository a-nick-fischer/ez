use weighted_trie::WeightedTrie;

pub struct Symbols(WeightedTrie);

impl Symbols {
    pub fn new<'a>(new_symbols: impl Iterator<Item = &'a String>) -> Self {
        let mut trie = WeightedTrie::new();
        
        for symbol in new_symbols {
            trie.insert(symbol.clone(), 1);
        }

        Symbols(trie)
    }

    pub fn search(&self, prefix: &str) -> Vec<String> {
        self.0.search(prefix)
    }
}