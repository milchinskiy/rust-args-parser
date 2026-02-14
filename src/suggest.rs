#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];
    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = core::cmp::min(core::cmp::min(curr[j] + 1, prev[j + 1] + 1), prev[j] + cost);
        }
        core::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}
