use std::collections::HashSet;

struct Difference<'first, 'second> {
    first_only: Vec<&'first str>,
    second_only: Vec<&'second str>
}

fn difference_iter<'a, 'b, 'ref_, T>(
    set_a: &'ref_ HashSet<&'a T>,
    set_b: &'ref_ HashSet<&'b T>,
) -> impl Iterator<Item = &'a T> + 'ref_
where
    T: std::cmp::Eq + std::hash::Hash + ?Sized,
    'a: 'ref_,
{
    set_a.iter().filter(move |a| !set_b.contains(*a)).copied()
    // set_b.iter().filter(move |b| !set_a.contains(*b)).copied()
}

fn find_difference<'fst, 'snd>(sentence1: &'fst str, sentence2: &'snd str) -> Difference<'fst, 'snd> 
{
    let sentence_1_words: HashSet<&str> = sentence1.split(" ").collect();
    let sentence_2_words: HashSet<&str> = sentence2.split(" ").collect();

    Difference {
        first_only: difference_iter(&sentence_1_words, &sentence_2_words).collect(),
        second_only: difference_iter(&sentence_2_words, &sentence_1_words).collect(),
    }
}

fn main() {
    let first_sentence = String::from("I love the surf and the sand.");
    let second_sentence = String::from("I hate the surf and the sand.");

    let first_only = {
        let third_sentence = String::from("I hate the snow and the sand.");
        let diff = find_difference(&first_sentence, &third_sentence);
        diff.first_only
    };

    assert_eq!(first_only, vec!["hate", "surf"]);

    let second_only = {
        let third_sentence = String::from("I hate the snow and the sand.");
        let diff = find_difference(&third_sentence, &second_sentence);
        diff.second_only
    };

    assert_eq!(second_only, vec!["snow"]);
}
