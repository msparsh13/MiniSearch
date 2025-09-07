use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};

pub struct TokenizerConfig {
    pub use_stemming: bool,
    pub min_ngram: Option<usize>,
    pub max_ngram: Option<usize>,
}

/**
 * TODO: ADD lemmatizing support
 * TODO: check whether we should get word of size <min_ngram
 * TODO: SET up condition like what if i want min_ngram and full word like if less than ngram size we get full word  and max_ngram and full word
 */
impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            use_stemming: false,
            min_ngram: None,
            max_ngram: None,
        }
    }
}

pub struct Tokenizer {
    config: TokenizerConfig,
    stemmer: Option<Stemmer>,
}

impl Tokenizer {
    pub fn new(config: TokenizerConfig) -> Self {
          if let (Some(min_n), Some(max_n)) = (config.min_ngram, config.max_ngram) {
            if min_n > max_n {
                 panic!("min_ngram should be <= max_ngram");
            }
        }
        let stemmer = if config.use_stemming {
            Some(Stemmer::create(Algorithm::English))
        } else {
            None
        };
        Self { config, stemmer }
    }

    pub fn tokenize(&self, text: &str, allow_ngram: bool) -> (Vec<String>, Option<Vec<String>>) {
        // 1. Word split
        let re = Regex::new(r"[A-Za-z0-9]+").unwrap();
        let mut words: Vec<String> = re
            .find_iter(&text.to_lowercase())
            .map(|m| m.as_str().to_string())
            .collect();

        // 2. Optional stemming
        if let Some(stemmer) = &self.stemmer {
            words = words
                .into_iter()
                .map(|w| stemmer.stem(&w).into_owned())
                .collect();
        }

        // 4. N-gram tokens
        // we are keeping words in another inverted index so we dont need to save full words here 
        let ngram_tokens = if allow_ngram {
            let min_n = self.config.min_ngram.unwrap_or(1);
            let max_n = self.config.max_ngram.unwrap_or(min_n);

            let mut ngrams = Vec::new();
            for word in &words {
                for n in min_n..=max_n {
                    ngrams.extend(Self::ngram_tokenize(word, n));
                }
            }
            Some(ngrams)
        } else {
            None
        };

        (words, ngram_tokens)
    }

    fn ngram_tokenize(word: &str, n: usize) -> Vec<String> {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() < n {
            return vec![];
        }
        chars.windows(n).map(|w| w.iter().collect()).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::index::tokenizer;

    use super::*;

    #[test]
    fn test_tokenize_words_only() {
        let tokenizer = Tokenizer::new(TokenizerConfig::default());
        let (words, ngrams) = tokenizer.tokenize("Hello World 123", false);

        // Words should be lowercased and split correctly
        assert_eq!(words, vec!["hello", "world", "123"]);

        // N-grams should be None because allow_ngram = false
        assert!(ngrams.is_none());
    }

    #[test]
    fn test_tokenize_with_ngrams() {
        let config = TokenizerConfig {
            use_stemming: false,
            min_ngram: Some(2),
            max_ngram: Some(3),
        };
        let tokenizer = Tokenizer::new(config);
        let (_words, ngrams) = tokenizer.tokenize("abc", true);

        let ngrams = ngrams.unwrap();

        // Check all 2-grams and 3-grams
        let expected: Vec<String> = vec!["ab", "bc", "abc"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        for gram in expected {
            assert!(ngrams.contains(&gram));
        }
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokenizer = Tokenizer::new(TokenizerConfig::default());
        let (words, ngrams) = tokenizer.tokenize("", true);

        assert!(words.is_empty());
        assert!(ngrams.unwrap().is_empty());
    }

    #[test]
     #[should_panic(expected = "min_ngram should be <= max_ngram")]
    fn min_ngram_more_than_max_ngram_should_fail() {
        let config = TokenizerConfig {
            use_stemming: false,
            min_ngram: Some(5),
            max_ngram: Some(3),
        };

        let result = Tokenizer::new(config);
    }


#[test]
fn one_min_or_max_n_gram_is_given() {
    let config = TokenizerConfig {
        use_stemming: false,
        min_ngram:Some(5),
        max_ngram: None,
    };

    let tokenizer = Tokenizer::new(config); // returns Tokenizer
    let (words, ngrams) = tokenizer.tokenize("Hello Worlds 123", true);

    assert_eq!(words, vec!["hello", "worlds" , "123"]);

    let ngrams = ngrams.unwrap();
    assert_eq!(ngrams , vec!["hello" ,"world" , "orlds" ]);
    for word in &words {
        if word.len() >= 5 {
            assert!(ngrams.iter().any(|g| g.len() == 5));
        }
    }
}

}
