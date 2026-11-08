use crate::engine::catalog::get_catalog;
use crate::model::Ecosystem;

/// Result of a typosquatting search
#[derive(Debug, Clone, PartialEq)]
pub struct TypoMatch {
    pub suggested_name: String,
    pub distance: usize,
    pub similarity: f64,
}

/// Computes the Damerau-Levenshtein distance (including adjacent character transpositions)
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut d = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m {
        d[i][0] = i;
    }
    for j in 0..=n {
        d[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };

            let mut min_val = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);

            // Adjacent transposition (Damerau condition)
            if i > 1
                && j > 1
                && a_chars[i - 1] == b_chars[j - 2]
                && a_chars[i - 2] == b_chars[j - 1]
            {
                min_val = min_val.min(d[i - 2][j - 2] + 1);
            }

            d[i][j] = min_val;
        }
    }

    d[m][n]
}

/// Computes normalized similarity between 0.0 and 1.0
pub fn similarity_ratio(a: &str, b: &str) -> f64 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let dist = levenshtein_distance(a, b);
    1.0 - (dist as f64 / max_len as f64)
}

/// Checks if a package name is a typo or typosquatting candidate of a popular package
pub fn find_typosquatting(ecosystem: Ecosystem, name: &str) -> Option<TypoMatch> {
    let lower_name = name.to_ascii_lowercase();
    let catalog = get_catalog(ecosystem);

    let mut best_match: Option<TypoMatch> = None;

    for &catalog_pkg in catalog {
        let catalog_lower = catalog_pkg.to_ascii_lowercase();

        // Exact match is not a typo
        if lower_name == catalog_lower {
            return None;
        }

        let dist = levenshtein_distance(&lower_name, &catalog_lower);
        let sim = similarity_ratio(&lower_name, &catalog_lower);

        let is_suspicious = if dist == 1 && (lower_name.len() >= 3 || catalog_lower.len() >= 3) {
            true
        } else if dist == 2 && lower_name.len() >= 5 && sim >= 0.70 {
            true
        } else {
            false
        };

        if is_suspicious {
            match &best_match {
                Some(prev) => {
                    if dist < prev.distance || (dist == prev.distance && sim > prev.similarity) {
                        best_match = Some(TypoMatch {
                            suggested_name: catalog_pkg.to_string(),
                            distance: dist,
                            similarity: sim,
                        });
                    }
                }
                None => {
                    best_match = Some(TypoMatch {
                        suggested_name: catalog_pkg.to_string(),
                        distance: dist,
                        similarity: sim,
                    });
                }
            }
        }
    }

    best_match
}
