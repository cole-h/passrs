use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
/// The ```ClosestMatch``` struct stores informations about the dictionary of known words
/// and the different sizes for the bags of words.
pub struct ClosestMatch {
    substrings: HashMap<String, HashSet<String>>,
    substring_sizes: Vec<usize>,
}

#[derive(Debug)]
struct SplitWord {
    word: String,
    substrings: HashSet<String>,
}

#[derive(Debug)]
struct ScoreValue {
    word: String,
    score: f32,
}

fn split_word<S, V>(word: S, sizes: V) -> SplitWord
where
    V: AsRef<[usize]>,
    S: AsRef<str>,
{
    let sizes = sizes.as_ref();
    let word = word.as_ref();
    let mut substrings: HashSet<String> = HashSet::new();
    for size in sizes {
        if *size > word.len() {
            continue;
        }
        for x in 0..(word.len() - size + 1) {
            let sub = word[x..(x + size)].to_string().to_lowercase();
            substrings.insert(sub);
        }
    }
    return SplitWord {
        word: word.to_owned(),
        substrings: substrings,
    };
}

fn evaluate(
    word_subs: &HashSet<String>,
    possible: String,
    possible_subs: &HashSet<String>,
) -> ScoreValue {
    let mut count = 0;
    let len_sum = word_subs.len() + possible_subs.len();
    for sub in word_subs {
        if possible_subs.contains(sub) {
            count += 1;
        }
    }
    let score = (count as f32) / (len_sum as f32);
    return ScoreValue {
        word: possible,
        score: score,
    };
}

// fn max_score(a: ScoreValue, b: ScoreValue) -> ScoreValue {
//     if a.score <= b.score {
//         return b;
//     }
//     return a;
// }

fn max_score(a: f32, b: f32) -> std::cmp::Ordering {
    if a <= b {
        return std::cmp::Ordering::Less;
    }
    return std::cmp::Ordering::Greater;
}

impl ClosestMatch {
    /// The function ```new``` takes a dictionary of known words with type ```Vec<String>``` and the
    /// different sizes of bag of words with type ```Vec<usize>```.
    /// It returns a ClosestMatch object.
    pub fn new<V>(dictionary: Vec<String>, sizes: V) -> ClosestMatch
    where
        V: AsRef<[usize]>,
    {
        let sizes = sizes.as_ref();
        let mut substrings: HashMap<String, HashSet<String>> = HashMap::new();
        let splitwords: Vec<SplitWord> = dictionary
            .iter()
            .map(|possible| split_word(possible.to_lowercase(), &sizes))
            .collect();
        for splitword in splitwords {
            substrings.insert(splitword.word, splitword.substrings);
        }
        return ClosestMatch {
            substrings: substrings,
            substring_sizes: sizes.to_vec(),
        };
    }

    /// The function ```get_closest``` takes a word with type ```String``` and
    /// returns the closest word in the dictionary of known words.
    pub fn get_closest<S>(&self, word: S) -> Option<String>
    where
        S: AsRef<str>,
    {
        let word = word.as_ref();
        let word_subs = split_word(&word, &self.substring_sizes).substrings;
        let best = self
            .substrings
            .iter()
            .map(|(possible, possible_subs)| {
                evaluate(&word_subs, possible.to_lowercase(), possible_subs)
            })
            .max_by(|a, b| max_score(a.score, b.score));
        match best {
            Some(expr) => Some(expr.word),
            None => None,
        }
    }

    pub fn get_closest_n<S>(&self, word: S, n: usize) -> Vec<String>
    where
        S: AsRef<str>,
    {
        let mut matches = Vec::with_capacity(n);
        let word = word.as_ref();
        let word_subs = split_word(&word, &self.substring_sizes).substrings;
        for _ in 0..n {
            let best = self
                .substrings
                .iter()
                .map(|(possible, possible_subs)| {
                    evaluate(&word_subs, possible.to_lowercase(), possible_subs)
                })
                .max_by(|a, b| max_score(a.score, b.score));
            match best {
                Some(expr) => matches.push(expr.word),
                None => continue,
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::ClosestMatch;

    #[test]
    fn it_works() {
        let cm = ClosestMatch::new(
            [
                "hello".to_string(),
                "bullo".to_string(),
                "hello world".to_string(),
            ]
            .to_vec(),
            [1, 2, 3].to_vec(),
        );
        let closest = cm.get_closest("hlo".to_string());
        println!("{:?}", closest);
    }
}
