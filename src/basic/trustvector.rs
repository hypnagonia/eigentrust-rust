use crate::sparse::entry::{Entry};


// CanonicalizeTrustVector canonicalizes the trust vector in-place,
// scaling it so that the elements sum to one,
// or making it a uniform vector that sums to one if it's a zero vector.

pub fn canonicalize_trust_vector(v: &mut Vec<Entry>) {
    if canonicalize(v).is_err() {
        let dim = v.len();
        let c = 1.0 / dim as f64;
        v.clear();
        for i in 0..dim {
            v.push(Entry {
                index: i,
                value: c,
            });
        }
    }
}

// Helper function to canonicalize a vector in-place.
// Returns an error if the vector is a zero vector.
fn canonicalize(entries: &mut Vec<Entry>) -> Result<(), &'static str> {
    let sum: f64 = entries.iter().map(|entry| entry.value).sum();
    
    if sum == 0.0 {
        return Err("Zero sum vector");
    }

    for entry in entries.iter_mut() {
        entry.value /= sum;
    }

    Ok(())
}