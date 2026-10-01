pub fn l2_normalize(mut vector: Vec<f32>) -> Vec<f32> {
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 1e-12 {
        for value in vector.iter_mut() {
            *value /= norm;
        }
    }
    vector
}

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    1.0 - dot.clamp(-1.0, 1.0)
}

fn mean_embedding(indices: &[usize], embeddings: &[Vec<f32>]) -> Vec<f32> {
    if indices.is_empty() {
        return Vec::new();
    }

    let dim = embeddings[indices[0]].len();
    let mut mean = vec![0f32; dim];
    for &index in indices {
        for (position, value) in embeddings[index].iter().enumerate() {
            mean[position] += value;
        }
    }

    let scale = 1.0 / indices.len() as f32;
    for value in mean.iter_mut() {
        *value *= scale;
    }

    l2_normalize(mean)
}

pub fn cluster(embeddings: &[Vec<f32>], threshold: f32) -> Vec<usize> {
    let count = embeddings.len();
    if count == 0 {
        return Vec::new();
    }

    let mut clusters: Vec<Vec<usize>> = (0..count).map(|index| vec![index]).collect();
    let mut centroids: Vec<Vec<f32>> = embeddings.to_vec();

    loop {
        let mut best: Option<(usize, usize, f32)> = None;

        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let distance = cosine_distance(&centroids[i], &centroids[j]);
                if best.map_or(true, |(_, _, current)| distance < current) {
                    best = Some((i, j, distance));
                }
            }
        }

        match best {
            Some((i, j, distance)) if distance <= threshold => {
                let merged = clusters.remove(j);
                centroids.remove(j);
                clusters[i].extend(merged);
                centroids[i] = mean_embedding(&clusters[i], embeddings);
            }
            _ => break,
        }
    }

    let mut labels = vec![0usize; count];
    for (label, cluster) in clusters.iter().enumerate() {
        for &index in cluster {
            labels[index] = label;
        }
    }

    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_to_unit_length() {
        let normalized = l2_normalize(vec![3.0, 4.0]);
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn zero_vector_stays_zero() {
        assert_eq!(l2_normalize(vec![0.0, 0.0]), vec![0.0, 0.0]);
    }

    #[test]
    fn empty_input_has_no_labels() {
        assert!(cluster(&[], 0.5).is_empty());
    }

    #[test]
    fn single_embedding_is_one_speaker() {
        assert_eq!(cluster(&[vec![1.0, 0.0]], 0.5), vec![0]);
    }

    #[test]
    fn separates_two_distinct_groups() {
        let embeddings = vec![
            l2_normalize(vec![1.0, 0.0, 0.0]),
            l2_normalize(vec![0.98, 0.02, 0.0]),
            l2_normalize(vec![0.0, 1.0, 0.0]),
            l2_normalize(vec![0.0, 0.99, 0.01]),
        ];

        let labels = cluster(&embeddings, 0.3);

        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[2], labels[3]);
        assert_ne!(labels[0], labels[2]);
    }

    #[test]
    fn high_threshold_merges_all() {
        let embeddings = vec![
            l2_normalize(vec![1.0, 0.0, 0.0]),
            l2_normalize(vec![0.0, 1.0, 0.0]),
        ];
        let labels = cluster(&embeddings, 1.5);
        assert_eq!(labels[0], labels[1]);
    }
}
